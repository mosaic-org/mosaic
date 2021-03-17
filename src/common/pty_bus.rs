use ::async_std::fs::File;
use ::async_std::prelude::*;
use ::async_std::stream::*;
use ::async_std::task;
use ::async_std::task::*;
use ::std::collections::HashMap;
use ::std::os::unix::io::{FromRawFd, RawFd};
use ::std::pin::*;
use ::std::sync::mpsc::Receiver;
use ::std::time::{Duration, Instant};
use ::vte;
use std::path::PathBuf;

use super::{ScreenInstruction, SenderWithContext, OPENCALLS};
use crate::os_input_output::OsApi;
use crate::utils::logging::debug_to_file;
use crate::{
    errors::{ContextType, ErrorContext},
    panes::PaneId,
};
use crate::{layout::Layout, wasm_vm::PluginInstruction};

struct ReadFromPid {
    pid: RawFd,
    os_input: Box<dyn OsApi>,
    file: Option<File>,
}

impl ReadFromPid {
    fn new(pid: &RawFd, os_input: Box<dyn OsApi>) -> ReadFromPid {
        let file;
        if cfg!(not(test)) {
            let std_file = unsafe { std::fs::File::from_raw_fd(*pid) };
            file = Some(File::from(std_file));
        } else {
            file = None;
        }
        ReadFromPid {
            pid: *pid,
            os_input,
            file,
        }
    }

    async fn next(&mut self) -> Option<Vec<u8>> {
        let mut buf = [0; 65535];
        #[cfg(not(test))]
        let data = self.file.as_ref().unwrap().read(&mut buf).await;
        #[cfg(test)]
        let data = self.os_input.read_from_tty_stdout(self.pid, &mut buf);
        match data {
            Ok(bytes) => {
                if bytes == 0 {
                    Some(vec![])
                } else {
                    Some(buf[..bytes].to_vec())
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub enum VteEvent {
    // TODO: try not to allocate Vecs
    Print(char),
    Execute(u8),                         // byte
    Hook(Vec<i64>, Vec<u8>, bool, char), // params, intermediates, ignore, char
    Put(u8),                             // byte
    Unhook,
    OscDispatch(Vec<Vec<u8>>, bool), // params, bell_terminated
    CsiDispatch(Vec<i64>, Vec<u8>, bool, char), // params, intermediates, ignore, char
    EscDispatch(Vec<u8>, bool, u8),  // intermediates, ignore, byte
}

struct VteEventSender {
    id: RawFd,
    sender: SenderWithContext<ScreenInstruction>,
}

impl VteEventSender {
    pub fn new(id: RawFd, sender: SenderWithContext<ScreenInstruction>) -> Self {
        VteEventSender { id, sender }
    }
}

impl vte::Perform for VteEventSender {
    fn print(&mut self, c: char) {
        let _ = self
            .sender
            .send(ScreenInstruction::Pty(self.id, VteEvent::Print(c)));
    }
    fn execute(&mut self, byte: u8) {
        let _ = self
            .sender
            .send(ScreenInstruction::Pty(self.id, VteEvent::Execute(byte)));
    }

    fn hook(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, c: char) {
        let params = params.iter().copied().collect();
        let intermediates = intermediates.iter().copied().collect();
        let instruction =
            ScreenInstruction::Pty(self.id, VteEvent::Hook(params, intermediates, ignore, c));
        let _ = self.sender.send(instruction);
    }

    fn put(&mut self, byte: u8) {
        let _ = self
            .sender
            .send(ScreenInstruction::Pty(self.id, VteEvent::Put(byte)));
    }

    fn unhook(&mut self) {
        let _ = self
            .sender
            .send(ScreenInstruction::Pty(self.id, VteEvent::Unhook));
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        let params = params.iter().map(|p| p.to_vec()).collect();
        let instruction =
            ScreenInstruction::Pty(self.id, VteEvent::OscDispatch(params, bell_terminated));
        let _ = self.sender.send(instruction);
    }

    fn csi_dispatch(&mut self, params: &[i64], intermediates: &[u8], ignore: bool, c: char) {
        let params = params.iter().copied().collect();
        let intermediates = intermediates.iter().copied().collect();
        let instruction = ScreenInstruction::Pty(
            self.id,
            VteEvent::CsiDispatch(params, intermediates, ignore, c),
        );
        let _ = self.sender.send(instruction);
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], ignore: bool, byte: u8) {
        let intermediates = intermediates.iter().copied().collect();
        let instruction =
            ScreenInstruction::Pty(self.id, VteEvent::EscDispatch(intermediates, ignore, byte));
        let _ = self.sender.send(instruction);
    }
}

/// Instructions related to PTYs (pseudoterminals).
#[derive(Clone, Debug)]
pub enum PtyInstruction {
    SpawnTerminal(Option<PathBuf>),
    SpawnTerminalVertically(Option<PathBuf>),
    SpawnTerminalHorizontally(Option<PathBuf>),
    NewTab,
    ClosePane(PaneId),
    CloseTab(Vec<PaneId>),
    Quit,
}

pub struct PtyBus {
    pub send_screen_instructions: SenderWithContext<ScreenInstruction>,
    pub send_plugin_instructions: SenderWithContext<PluginInstruction>,
    pub receive_pty_instructions: Receiver<(PtyInstruction, ErrorContext)>,
    pub id_to_child_pid: HashMap<RawFd, RawFd>,
    os_input: Box<dyn OsApi>,
    debug_to_file: bool,
    task_handles: HashMap<RawFd, JoinHandle<()>>,
}

/*
 * This function is a bit messy to support the integration tests.
 * The tests feed inputs through an os_input shim. At the moment,
 * async trait functions are not possible so there's no way to
 * have two different async implementations in the OsApi trait.
 * As a result, we have to add the abstraction here instead
 * of in the test. So ReadFromPid implements stream to support
 * tests, and then this function does user space polling of the stream.
 * The production code just does a read on an async_std::fs::File.
 */
fn stream_terminal_bytes(
    pid: RawFd,
    mut send_screen_instructions: SenderWithContext<ScreenInstruction>,
    os_input: Box<dyn OsApi>,
    debug: bool,
) -> JoinHandle<()> {
    let mut err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());
    task::spawn({
        async move {
            err_ctx.add_call(ContextType::AsyncTask);
            send_screen_instructions.update(err_ctx);
            let mut vte_parser = vte::Parser::new();
            let mut vte_event_sender = VteEventSender::new(pid, send_screen_instructions.clone());
            let mut terminal_bytes = ReadFromPid::new(&pid, os_input);

            while let Some(bytes) = terminal_bytes.next().await {
                let bytes_is_empty = bytes.is_empty();
                for byte in bytes {
                    if debug {
                        debug_to_file(byte, pid).unwrap();
                    }
                    vte_parser.advance(&mut vte_event_sender, byte);
                }
                if cfg!(not(test)) {
                    if !bytes_is_empty {
                        let _ = send_screen_instructions.send(ScreenInstruction::Render);
                        task::sleep(::std::time::Duration::from_millis(10)).await;
                    }
                } else {
                    // the tests do a non blocking synchronous read on the fd.
                    // put them to sleep for a little while
                    if !bytes_is_empty {
                        let _ = send_screen_instructions.send(ScreenInstruction::Render);
                    }
                    task::sleep(::std::time::Duration::from_millis(30)).await;
                }
            }
            send_screen_instructions
                .send(ScreenInstruction::Render)
                .unwrap();
            #[cfg(not(test))]
            // this is a little hacky, and is because the tests end the file as soon as
            // we read everything, rather than hanging until there is new data
            // a better solution would be to fix the test fakes, but this will do for now
            send_screen_instructions
                .send(ScreenInstruction::ClosePane(PaneId::Terminal(pid)))
                .unwrap();
        }
    })
}

impl PtyBus {
    pub fn new(
        receive_pty_instructions: Receiver<(PtyInstruction, ErrorContext)>,
        send_screen_instructions: SenderWithContext<ScreenInstruction>,
        send_plugin_instructions: SenderWithContext<PluginInstruction>,
        os_input: Box<dyn OsApi>,
        debug_to_file: bool,
    ) -> Self {
        PtyBus {
            send_screen_instructions,
            send_plugin_instructions,
            receive_pty_instructions,
            os_input,
            id_to_child_pid: HashMap::new(),
            debug_to_file,
            task_handles: HashMap::new(),
        }
    }
    pub fn spawn_terminal(&mut self, file_to_open: Option<PathBuf>) -> RawFd {
        let (pid_primary, pid_secondary): (RawFd, RawFd) =
            self.os_input.spawn_terminal(file_to_open);
        let task_handle = stream_terminal_bytes(
            pid_primary,
            self.send_screen_instructions.clone(),
            self.os_input.clone(),
            self.debug_to_file,
        );
        self.task_handles.insert(pid_primary, task_handle);
        self.id_to_child_pid.insert(pid_primary, pid_secondary);
        pid_primary
    }
    pub fn spawn_terminals_for_layout(&mut self, layout: Layout) {
        let total_panes = layout.total_terminal_panes();
        let mut new_pane_pids = vec![];
        for _ in 0..total_panes {
            let (pid_primary, pid_secondary): (RawFd, RawFd) = self.os_input.spawn_terminal(None);
            self.id_to_child_pid.insert(pid_primary, pid_secondary);
            new_pane_pids.push(pid_primary);
        }
        self.send_screen_instructions
            .send(ScreenInstruction::ApplyLayout((
                layout,
                new_pane_pids.clone(),
            )))
            .unwrap();
        for id in new_pane_pids {
            let task_handle = stream_terminal_bytes(
                id,
                self.send_screen_instructions.clone(),
                self.os_input.clone(),
                self.debug_to_file,
            );
            self.task_handles.insert(id, task_handle);
        }
    }
    pub fn close_pane(&mut self, id: PaneId) {
        match id {
            PaneId::Terminal(id) => {
                let child_pid = self.id_to_child_pid.remove(&id).unwrap();
                let handle = self.task_handles.remove(&id).unwrap();
                self.os_input.kill(child_pid).unwrap();
                task::block_on(async {
                    handle.cancel().await;
                });
            }
            PaneId::Plugin(pid) => drop(
                self.send_plugin_instructions
                    .send(PluginInstruction::Unload(pid)),
            ),
        }
    }
    pub fn close_tab(&mut self, ids: Vec<PaneId>) {
        ids.iter().for_each(|&id| {
            self.close_pane(id);
        });
    }
}

impl Drop for PtyBus {
    fn drop(&mut self) {
        let child_ids: Vec<RawFd> = self.id_to_child_pid.keys().copied().collect();
        for id in child_ids {
            self.close_pane(PaneId::Terminal(id));
        }
    }
}
