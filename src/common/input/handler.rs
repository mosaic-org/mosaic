//! Main input logic.

use super::keybinds::Keybinds;
use crate::common::input::config::Config;
use crate::common::{AppInstruction, SenderWithContext, OPENCALLS};
use crate::errors::ContextType;
use crate::os_input_output::OsApi;
use crate::pty_bus::PtyInstruction;
use crate::screen::ScreenInstruction;
use crate::wasm_vm::PluginInstruction;
use crate::CommandIsExecuting;

use termion::input::{TermRead, TermReadEventsAndRaw};
use zellij_tile::data::{Action, Direction, Event, InputMode, Key, ModeInfo};

/// Handles the dispatching of [`Action`]s according to the current
/// [`InputMode`], and keep tracks of the current [`InputMode`].
struct InputHandler {
    /// The current input mode
    mode: InputMode,
    os_input: Box<dyn OsApi>,
    config: Config,
    command_is_executing: CommandIsExecuting,
    send_screen_instructions: SenderWithContext<ScreenInstruction>,
    send_pty_instructions: SenderWithContext<PtyInstruction>,
    send_plugin_instructions: SenderWithContext<PluginInstruction>,
    send_app_instructions: SenderWithContext<AppInstruction>,
    should_exit: bool,
}

impl InputHandler {
    /// Returns a new [`InputHandler`] with the attributes specified as arguments.
    fn new(
        os_input: Box<dyn OsApi>,
        command_is_executing: CommandIsExecuting,
        config: Config,
        send_screen_instructions: SenderWithContext<ScreenInstruction>,
        send_pty_instructions: SenderWithContext<PtyInstruction>,
        send_plugin_instructions: SenderWithContext<PluginInstruction>,
        send_app_instructions: SenderWithContext<AppInstruction>,
    ) -> Self {
        InputHandler {
            mode: InputMode::Normal,
            os_input,
            config,
            command_is_executing,
            send_screen_instructions,
            send_pty_instructions,
            send_plugin_instructions,
            send_app_instructions,
            should_exit: false,
        }
    }

    /// Main input event loop. Interprets the terminal [`Event`](termion::event::Event)s
    /// as [`Action`]s according to the current [`InputMode`], and dispatches those actions.
    fn handle_input(&mut self) {
        let mut err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());
        err_ctx.add_call(ContextType::StdinHandler);
        let alt_left_bracket = vec![27, 91];
        loop {
            if self.should_exit {
                break;
            }
            let stdin_buffer = self.os_input.read_from_stdin();
            for key_result in stdin_buffer.events_and_raw() {
                match key_result {
                    Ok((event, raw_bytes)) => match event {
                        termion::event::Event::Key(key) => {
                            let key = cast_termion_key(key);
                            self.handle_key(&key, raw_bytes);
                        }
                        termion::event::Event::Unsupported(unsupported_key) => {
                            // we have to do this because of a bug in termion
                            // this should be a key event and not an unsupported event
                            if unsupported_key == alt_left_bracket {
                                let key = Key::Alt('[');
                                self.handle_key(&key, raw_bytes);
                            }
                        }
                        termion::event::Event::Mouse(_) => {
                            // Mouse events aren't implemented yet,
                            // use a NoOp untill then.
                        }
                    },
                    Err(err) => panic!("Encountered read error: {:?}", err),
                }
            }
        }
    }
    fn handle_key(&mut self, key: &Key, raw_bytes: Vec<u8>) {
        let keybinds = &self.config.keybinds;
        for action in Keybinds::key_to_actions(&key, raw_bytes, &self.mode, keybinds) {
            let should_exit = self.dispatch_action(action);
            if should_exit {
                self.should_exit = true;
            }
        }
    }

    /// Dispatches an [`Action`].
    ///
    /// This function's body dictates what each [`Action`] actually does when
    /// dispatched.
    ///
    /// # Return value
    /// Currently, this function returns a boolean that indicates whether
    /// [`Self::handle_input()`] should break after this action is dispatched.
    /// This is a temporary measure that is only necessary due to the way that the
    /// framework works, and shouldn't be necessary anymore once the test framework
    /// is revised. See [issue#183](https://github.com/zellij-org/zellij/issues/183).
    fn dispatch_action(&mut self, action: Action) -> bool {
        let mut should_break = false;

        match action {
            Action::Write(val) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ClearScroll)
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::WriteCharacter(val))
                    .unwrap();
            }
            Action::Quit => {
                self.exit();
                should_break = true;
            }
            Action::SwitchToMode(mode) => {
                self.mode = mode;
                let keybinds: Vec<(Key, Vec<Action>)> = self
                    .config
                    .keybinds
                    .0
                    .get(&mode)
                    .cloned()
                    .unwrap_or_else(|| Keybinds::get_defaults_for_mode(&mode))
                    .0
                    .into_iter()
                    .collect();
                self.send_plugin_instructions
                    .send(PluginInstruction::Update(
                        None,
                        Event::ModeUpdate(ModeInfo {
                            mode,
                            keybinds: keybinds.clone(),
                        }),
                    ))
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::ChangeMode(ModeInfo { mode, keybinds }))
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::Render)
                    .unwrap();
            }
            Action::Resize(direction) => {
                let screen_instr = match direction {
                    Direction::Left => ScreenInstruction::ResizeLeft,
                    Direction::Right => ScreenInstruction::ResizeRight,
                    Direction::Up => ScreenInstruction::ResizeUp,
                    Direction::Down => ScreenInstruction::ResizeDown,
                };
                self.send_screen_instructions.send(screen_instr).unwrap();
            }
            Action::SwitchFocus => {
                self.send_screen_instructions
                    .send(ScreenInstruction::SwitchFocus)
                    .unwrap();
            }
            Action::FocusNextPane => {
                self.send_screen_instructions
                    .send(ScreenInstruction::FocusNextPane)
                    .unwrap();
            }
            Action::FocusPreviousPane => {
                self.send_screen_instructions
                    .send(ScreenInstruction::FocusPreviousPane)
                    .unwrap();
            }
            Action::MoveFocus(direction) => {
                let screen_instr = match direction {
                    Direction::Left => ScreenInstruction::MoveFocusLeft,
                    Direction::Right => ScreenInstruction::MoveFocusRight,
                    Direction::Up => ScreenInstruction::MoveFocusUp,
                    Direction::Down => ScreenInstruction::MoveFocusDown,
                };
                self.send_screen_instructions.send(screen_instr).unwrap();
            }
            Action::ScrollUp => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ScrollUp)
                    .unwrap();
            }
            Action::ScrollDown => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ScrollDown)
                    .unwrap();
            }
            Action::PageScrollUp => {
                self.send_screen_instructions
                    .send(ScreenInstruction::PageScrollUp)
                    .unwrap();
            }
            Action::PageScrollDown => {
                self.send_screen_instructions
                    .send(ScreenInstruction::PageScrollDown)
                    .unwrap();
            }
            Action::ToggleFocusFullscreen => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ToggleActiveTerminalFullscreen)
                    .unwrap();
            }
            Action::NewPane(direction) => {
                let pty_instr = match direction {
                    Some(Direction::Left) => PtyInstruction::SpawnTerminalVertically(None),
                    Some(Direction::Right) => PtyInstruction::SpawnTerminalVertically(None),
                    Some(Direction::Up) => PtyInstruction::SpawnTerminalHorizontally(None),
                    Some(Direction::Down) => PtyInstruction::SpawnTerminalHorizontally(None),
                    // No direction specified - try to put it in the biggest available spot
                    None => PtyInstruction::SpawnTerminal(None),
                };
                self.command_is_executing.opening_new_pane();
                self.send_pty_instructions.send(pty_instr).unwrap();
                self.command_is_executing.wait_until_new_pane_is_opened();
            }
            Action::CloseFocus => {
                self.command_is_executing.closing_pane();
                self.send_screen_instructions
                    .send(ScreenInstruction::CloseFocusedPane)
                    .unwrap();
                self.command_is_executing.wait_until_pane_is_closed();
            }
            Action::NewTab => {
                self.command_is_executing.opening_new_pane();
                self.send_pty_instructions
                    .send(PtyInstruction::NewTab)
                    .unwrap();
                self.command_is_executing.wait_until_new_pane_is_opened();
            }
            Action::GoToNextTab => {
                self.send_screen_instructions
                    .send(ScreenInstruction::SwitchTabNext)
                    .unwrap();
            }
            Action::GoToPreviousTab => {
                self.send_screen_instructions
                    .send(ScreenInstruction::SwitchTabPrev)
                    .unwrap();
            }
            Action::ToggleActiveSyncPanes => {
                self.send_screen_instructions
                    .send(ScreenInstruction::ToggleActiveSyncPanes)
                    .unwrap();
            }
            Action::CloseTab => {
                self.command_is_executing.closing_pane();
                self.send_screen_instructions
                    .send(ScreenInstruction::CloseTab)
                    .unwrap();
                self.command_is_executing.wait_until_pane_is_closed();
            }
            Action::GoToTab(i) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::GoToTab(i))
                    .unwrap();
            }
            Action::TabNameInput(c) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::UpdateTabName(c))
                    .unwrap();
            }
            Action::NoOp => {}
        }

        should_break
    }

    /// Routine to be called when the input handler exits (at the moment this is the
    /// same as quitting Zellij).
    fn exit(&mut self) {
        self.send_app_instructions
            .send(AppInstruction::Exit)
            .unwrap();
    }
}

/// Entry point to the module. Instantiates an [`InputHandler`] and starts
/// its [`InputHandler::handle_input()`] loop.
pub fn input_loop(
    os_input: Box<dyn OsApi>,
    config: Config,
    command_is_executing: CommandIsExecuting,
    send_screen_instructions: SenderWithContext<ScreenInstruction>,
    send_pty_instructions: SenderWithContext<PtyInstruction>,
    send_plugin_instructions: SenderWithContext<PluginInstruction>,
    send_app_instructions: SenderWithContext<AppInstruction>,
) {
    let _handler = InputHandler::new(
        os_input,
        command_is_executing,
        config,
        send_screen_instructions,
        send_pty_instructions,
        send_plugin_instructions,
        send_app_instructions,
    )
    .handle_input();
}

pub fn parse_keys(input_bytes: &[u8]) -> Vec<Key> {
    input_bytes.keys().flatten().map(cast_termion_key).collect()
}

// FIXME: This is an absolutely cursed function that should be destroyed as soon
// as an alternative that doesn't touch zellij-tile can be developed...
fn cast_termion_key(event: termion::event::Key) -> Key {
    match event {
        termion::event::Key::Backspace => Key::Backspace,
        termion::event::Key::Left => Key::Left,
        termion::event::Key::Right => Key::Right,
        termion::event::Key::Up => Key::Up,
        termion::event::Key::Down => Key::Down,
        termion::event::Key::Home => Key::Home,
        termion::event::Key::End => Key::End,
        termion::event::Key::PageUp => Key::PageUp,
        termion::event::Key::PageDown => Key::PageDown,
        termion::event::Key::BackTab => Key::BackTab,
        termion::event::Key::Delete => Key::Delete,
        termion::event::Key::Insert => Key::Insert,
        termion::event::Key::F(n) => Key::F(n),
        termion::event::Key::Char(c) => Key::Char(c),
        termion::event::Key::Alt(c) => Key::Alt(c),
        termion::event::Key::Ctrl(c) => Key::Ctrl(c),
        termion::event::Key::Null => Key::Null,
        termion::event::Key::Esc => Key::Esc,
        _ => {
            unimplemented!("Encountered an unknown key!")
        }
    }
}
