#[cfg(test)]
mod tests;

mod boundaries;
mod command_is_executing;
mod layout;
mod os_input_output;
mod pty_bus;
mod screen;
mod terminal_pane;
mod utils;

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use serde::{Deserialize, Serialize};
use serde_yaml;
use structopt::StructOpt;

use crate::command_is_executing::CommandIsExecuting;
use crate::layout::Layout;
use crate::os_input_output::{get_os_input, OsApi};
use crate::pty_bus::{PtyBus, PtyInstruction, VteEvent};
use crate::screen::{Screen, ScreenInstruction};
use crate::utils::{consts::MOSAIC_IPC_PIPE, logging::*};

#[derive(Serialize, Deserialize, Debug)]
enum ApiCommand {
    OpenFile(PathBuf),
    SplitHorizontally,
    SplitVertically,
    MoveFocus,
}

#[derive(StructOpt, Debug, Default)]
#[structopt(name = "mosaic")]
pub struct Opt {
    #[structopt(short, long)]
    /// Send "split (direction h == horizontal / v == vertical)" to active mosaic session
    split: Option<char>,
    #[structopt(short, long)]
    /// Send "move focused pane" to active mosaic session
    move_focus: bool,
    #[structopt(short, long)]
    /// Send "open file in new pane" to active mosaic session
    open_file: Option<PathBuf>,
    #[structopt(long)]
    /// Maximum panes on screen, caution: opening more panes will close old ones
    max_panes: Option<usize>,
    #[structopt(short, long)]
    /// Path to a layout yaml file
    layout: Option<PathBuf>,
    #[structopt(short, long)]
    debug: bool,
}

pub fn main() {
    let opts = Opt::from_args();
    if opts.split.is_some() {
        match opts.split {
            Some('h') => {
                let mut stream = UnixStream::connect(MOSAIC_IPC_PIPE).unwrap();
                let api_command = bincode::serialize(&ApiCommand::SplitHorizontally).unwrap();
                stream.write_all(&api_command).unwrap();
            }
            Some('v') => {
                let mut stream = UnixStream::connect(MOSAIC_IPC_PIPE).unwrap();
                let api_command = bincode::serialize(&ApiCommand::SplitVertically).unwrap();
                stream.write_all(&api_command).unwrap();
            }
            _ => {}
        };
    } else if opts.move_focus {
        let mut stream = UnixStream::connect(MOSAIC_IPC_PIPE).unwrap();
        let api_command = bincode::serialize(&ApiCommand::MoveFocus).unwrap();
        stream.write_all(&api_command).unwrap();
    } else if opts.open_file.is_some() {
        let mut stream = UnixStream::connect(MOSAIC_IPC_PIPE).unwrap();
        let file_to_open = opts.open_file.unwrap();
        let api_command = bincode::serialize(&ApiCommand::OpenFile(file_to_open)).unwrap();
        stream.write_all(&api_command).unwrap();
    } else {
        let os_input = get_os_input();
        start(Box::new(os_input), opts);
    }
}

pub enum AppInstruction {
    Exit,
}

pub fn start(mut os_input: Box<dyn OsApi>, opts: Opt) {
    let mut active_threads = vec![];

    let command_is_executing = CommandIsExecuting::new();

    delete_log_dir().unwrap();
    delete_log_file().unwrap();

    let full_screen_ws = os_input.get_terminal_size_using_fd(0);
    os_input.into_raw_mode(0);
    let (send_screen_instructions, receive_screen_instructions): (
        Sender<ScreenInstruction>,
        Receiver<ScreenInstruction>,
    ) = channel();
    let (send_pty_instructions, receive_pty_instructions): (
        Sender<PtyInstruction>,
        Receiver<PtyInstruction>,
    ) = channel();
    let (send_app_instructions, receive_app_instructions): (
        Sender<AppInstruction>,
        Receiver<AppInstruction>,
    ) = channel();
    let mut screen = Screen::new(
        receive_screen_instructions,
        send_pty_instructions.clone(),
        send_app_instructions.clone(),
        &full_screen_ws,
        os_input.clone(),
        opts.max_panes,
    );
    let mut pty_bus = PtyBus::new(
        receive_pty_instructions,
        send_screen_instructions.clone(),
        os_input.clone(),
        opts.debug,
    );

    active_threads.push(
        thread::Builder::new()
            .name("pty".to_string())
            .spawn({
                let mut command_is_executing = command_is_executing.clone();
                move || {
                    match opts.layout {
                        Some(layout_path) => {
                            use std::fs::File;
                            let mut layout_file = File::open(&layout_path)
                                .expect(&format!("cannot find layout {}", layout_path.display()));
                            let mut layout = String::new();
                            layout_file.read_to_string(&mut layout).expect(&format!(
                                "could not read layout {}",
                                layout_path.display()
                            ));
                            let layout: Layout = serde_yaml::from_str(&layout).expect(&format!(
                                "could not parse layout {}",
                                layout_path.display()
                            ));
                            pty_bus.spawn_terminals_for_layout(layout);
                        }
                        None => {
                            pty_bus.spawn_terminal_vertically(None);
                        }
                    }
                    loop {
                        let event = pty_bus
                            .receive_pty_instructions
                            .recv()
                            .expect("failed to receive event on channel");
                        match event {
                            PtyInstruction::SpawnTerminal(file_to_open) => {
                                pty_bus.spawn_terminal(file_to_open);
                            }
                            PtyInstruction::SpawnTerminalVertically(file_to_open) => {
                                pty_bus.spawn_terminal_vertically(file_to_open);
                            }
                            PtyInstruction::SpawnTerminalHorizontally(file_to_open) => {
                                pty_bus.spawn_terminal_horizontally(file_to_open);
                            }
                            PtyInstruction::ClosePane(id) => {
                                pty_bus.close_pane(id);
                                command_is_executing.done_closing_pane();
                            }
                            PtyInstruction::Quit => {
                                break;
                            }
                        }
                    }
                }
            })
            .unwrap(),
    );

    active_threads.push(
        thread::Builder::new()
            .name("screen".to_string())
            .spawn({
                let mut command_is_executing = command_is_executing.clone();
                move || loop {
                    let event = screen
                        .receiver
                        .recv()
                        .expect("failed to receive event on channel");
                    match event {
                        ScreenInstruction::Pty(pid, vte_event) => {
                            screen.handle_pty_event(pid, vte_event);
                        }
                        ScreenInstruction::Render => {
                            screen.render();
                        }
                        ScreenInstruction::NewPane(pid) => {
                            screen.new_pane(pid);
                            command_is_executing.done_opening_new_pane();
                        }
                        ScreenInstruction::HorizontalSplit(pid) => {
                            screen.horizontal_split(pid);
                            command_is_executing.done_opening_new_pane();
                        }
                        ScreenInstruction::VerticalSplit(pid) => {
                            screen.vertical_split(pid);
                            command_is_executing.done_opening_new_pane();
                        }
                        ScreenInstruction::WriteCharacter(bytes) => {
                            screen.write_to_active_terminal(bytes);
                        }
                        ScreenInstruction::ResizeLeft => {
                            screen.resize_left();
                        }
                        ScreenInstruction::ResizeRight => {
                            screen.resize_right();
                        }
                        ScreenInstruction::ResizeDown => {
                            screen.resize_down();
                        }
                        ScreenInstruction::ResizeUp => {
                            screen.resize_up();
                        }
                        ScreenInstruction::MoveFocus => {
                            screen.move_focus();
                        }
                        ScreenInstruction::ScrollUp => {
                            screen.scroll_active_terminal_up();
                        }
                        ScreenInstruction::ScrollDown => {
                            screen.scroll_active_terminal_down();
                        }
                        ScreenInstruction::ClearScroll => {
                            screen.clear_active_terminal_scroll();
                        }
                        ScreenInstruction::CloseFocusedPane => {
                            screen.close_focused_pane();
                        }
                        ScreenInstruction::ClosePane(id) => {
                            screen.close_pane(id);
                        }
                        ScreenInstruction::ToggleActiveTerminalFullscreen => {
                            screen.toggle_active_terminal_fullscreen();
                        }
                        ScreenInstruction::ApplyLayout((layout, new_pane_pids)) => {
                            screen.apply_layout(layout, new_pane_pids)
                        }
                        ScreenInstruction::Quit => {
                            break;
                        }
                    }
                }
            })
            .unwrap(),
    );

    // TODO: currently we don't push this into active_threads
    // because otherwise the app will hang. Need to fix this so it both
    // listens to the ipc-bus and is able to quit cleanly
    #[cfg(not(test))]
    let _ipc_thread = thread::Builder::new()
        .name("ipc_server".to_string())
        .spawn({
            let send_pty_instructions = send_pty_instructions.clone();
            let send_screen_instructions = send_screen_instructions.clone();
            move || {
                std::fs::remove_file(MOSAIC_IPC_PIPE).ok();
                let listener = std::os::unix::net::UnixListener::bind(MOSAIC_IPC_PIPE)
                    .expect("could not listen on ipc socket");

                for stream in listener.incoming() {
                    match stream {
                        Ok(mut stream) => {
                            let mut buffer = [0; 65535]; // TODO: more accurate
                            stream
                                .read(&mut buffer)
                                .expect("failed to parse ipc message");
                            let decoded: ApiCommand = bincode::deserialize(&buffer)
                                .expect("failed to deserialize ipc message");
                            match &decoded {
                                ApiCommand::OpenFile(file_name) => {
                                    let path = PathBuf::from(file_name);
                                    send_pty_instructions
                                        .send(PtyInstruction::SpawnTerminal(Some(path)))
                                        .unwrap();
                                }
                                ApiCommand::SplitHorizontally => {
                                    send_pty_instructions
                                        .send(PtyInstruction::SpawnTerminalHorizontally(None))
                                        .unwrap();
                                }
                                ApiCommand::SplitVertically => {
                                    send_pty_instructions
                                        .send(PtyInstruction::SpawnTerminalVertically(None))
                                        .unwrap();
                                }
                                ApiCommand::MoveFocus => {
                                    send_screen_instructions
                                        .send(ScreenInstruction::MoveFocus)
                                        .unwrap();
                                }
                            }
                        }
                        Err(err) => {
                            panic!("err {:?}", err);
                        }
                    }
                }
            }
        })
        .unwrap();

    let _stdin_thread = thread::Builder::new().name("stdin".to_string()).spawn({
        let send_screen_instructions = send_screen_instructions.clone();
        let send_pty_instructions = send_pty_instructions.clone();
        let send_app_instructions = send_app_instructions.clone();
        let os_input = os_input.clone();

        let mut command_is_executing = command_is_executing.clone();
        move || {
            let mut stdin = os_input.get_stdin_reader();
            loop {
                let mut buffer = [0; 10]; // TODO: more accurately
                stdin.read(&mut buffer).expect("failed to read stdin");
                // uncomment this to print the entered character to a log file (/tmp/mosaic/mosaic-log.txt) for debugging
                //crate::utils::logging::debug_log_to_file(format!("buffer {:?}", buffer));
                match buffer {
                    [10, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-j
                        send_screen_instructions
                            .send(ScreenInstruction::ResizeDown)
                            .unwrap();
                    }
                    [11, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-k
                        send_screen_instructions
                            .send(ScreenInstruction::ResizeUp)
                            .unwrap();
                    }
                    [16, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-p
                        send_screen_instructions
                            .send(ScreenInstruction::MoveFocus)
                            .unwrap();
                    }
                    [8, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-h
                        send_screen_instructions
                            .send(ScreenInstruction::ResizeLeft)
                            .unwrap();
                    }
                    [12, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-l
                        send_screen_instructions
                            .send(ScreenInstruction::ResizeRight)
                            .unwrap();
                    }
                    [26, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-z
                        command_is_executing.opening_new_pane();
                        send_pty_instructions
                            .send(PtyInstruction::SpawnTerminal(None))
                            .unwrap();
                        command_is_executing.wait_until_new_pane_is_opened();
                    }
                    [14, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-n
                        command_is_executing.opening_new_pane();
                        send_pty_instructions
                            .send(PtyInstruction::SpawnTerminalVertically(None))
                            .unwrap();
                        command_is_executing.wait_until_new_pane_is_opened();
                    }
                    [2, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-b
                        command_is_executing.opening_new_pane();
                        send_pty_instructions
                            .send(PtyInstruction::SpawnTerminalHorizontally(None))
                            .unwrap();
                        command_is_executing.wait_until_new_pane_is_opened();
                    }
                    [17, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-q
                        let _ = send_screen_instructions.send(ScreenInstruction::Quit);
                        let _ = send_pty_instructions.send(PtyInstruction::Quit);
                        let _ = send_app_instructions.send(AppInstruction::Exit);
                        break;
                    }
                    [27, 91, 53, 94, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-PgUp
                        send_screen_instructions
                            .send(ScreenInstruction::ScrollUp)
                            .unwrap();
                    }
                    [27, 91, 54, 94, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-PgDown
                        send_screen_instructions
                            .send(ScreenInstruction::ScrollDown)
                            .unwrap();
                    }
                    [24, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-x
                        command_is_executing.closing_pane();
                        send_screen_instructions
                            .send(ScreenInstruction::CloseFocusedPane)
                            .unwrap();
                        command_is_executing.wait_until_pane_is_closed();
                    }
                    [5, 0, 0, 0, 0, 0, 0, 0, 0, 0] => {
                        // ctrl-e
                        send_screen_instructions
                            .send(ScreenInstruction::ToggleActiveTerminalFullscreen)
                            .unwrap();
                    }
                    _ => {
                        send_screen_instructions
                            .send(ScreenInstruction::ClearScroll)
                            .unwrap();
                        send_screen_instructions
                            .send(ScreenInstruction::WriteCharacter(buffer))
                            .unwrap();
                    }
                }
            }
        }
    });

    loop {
        let app_instruction = receive_app_instructions
            .recv()
            .expect("failed to receive app instruction on channel");
        match app_instruction {
            AppInstruction::Exit => {
                let _ = send_screen_instructions.send(ScreenInstruction::Quit);
                let _ = send_pty_instructions.send(PtyInstruction::Quit);
                break;
            }
        }
    }

    for thread_handler in active_threads {
        thread_handler.join().unwrap();
    }
    // cleanup();
    let reset_style = "\u{1b}[m";
    let goto_start_of_last_line = format!("\u{1b}[{};{}H", full_screen_ws.rows, 1);
    let goodbye_message = format!(
        "{}\n{}Bye from Mosaic!",
        goto_start_of_last_line, reset_style
    );

    os_input.unset_raw_mode(0);
    os_input
        .get_stdout_writer()
        .write(goodbye_message.as_bytes())
        .unwrap();
    os_input.get_stdout_writer().flush().unwrap();
}
