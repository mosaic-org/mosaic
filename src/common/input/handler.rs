//! Main input logic.

use super::actions::Action;
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
use zellij_tile::data::{Event, InputMode, Key, ModeInfo, Palette};

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
                self.send_plugin_instructions
                    .send(PluginInstruction::Update(
                        None,
                        Event::ModeUpdate(get_mode_info(mode, self.os_input.load_palette())),
                    ))
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::ChangeMode(get_mode_info(
                        mode,
                        self.os_input.load_palette(),
                    )))
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::Render)
                    .unwrap();
            }
            Action::Resize(direction) => {
                let screen_instr = match direction {
                    super::actions::Direction::Left => ScreenInstruction::ResizeLeft,
                    super::actions::Direction::Right => ScreenInstruction::ResizeRight,
                    super::actions::Direction::Up => ScreenInstruction::ResizeUp,
                    super::actions::Direction::Down => ScreenInstruction::ResizeDown,
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
                    super::actions::Direction::Left => ScreenInstruction::MoveFocusLeft,
                    super::actions::Direction::Right => ScreenInstruction::MoveFocusRight,
                    super::actions::Direction::Up => ScreenInstruction::MoveFocusUp,
                    super::actions::Direction::Down => ScreenInstruction::MoveFocusDown,
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
                    Some(super::actions::Direction::Left) => {
                        PtyInstruction::SpawnTerminalVertically(None)
                    }
                    Some(super::actions::Direction::Right) => {
                        PtyInstruction::SpawnTerminalVertically(None)
                    }
                    Some(super::actions::Direction::Up) => {
                        PtyInstruction::SpawnTerminalHorizontally(None)
                    }
                    Some(super::actions::Direction::Down) => {
                        PtyInstruction::SpawnTerminalHorizontally(None)
                    }
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
            Action::SearchTabInput(c) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::UpdateTabSearch(c))
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

/// Creates a [`Help`] struct indicating the current [`InputMode`] and its keybinds
/// (as pairs of [`String`]s).
// TODO this should probably be automatically generated in some way
pub fn get_mode_info(mode: InputMode, palette: Palette) -> ModeInfo {
    let mut keybinds: Vec<(String, String)> = vec![];
    let mut input_prompt: String = String::new();
    match mode {
        InputMode::Normal | InputMode::Locked => {}
        InputMode::Resize => {
            keybinds.push(("←↓↑→".to_string(), "Resize".to_string()));
        }
        InputMode::Pane => {
            keybinds.push(("←↓↑→".to_string(), "Move focus".to_string()));
            keybinds.push(("p".to_string(), "Next".to_string()));
            keybinds.push(("n".to_string(), "New".to_string()));
            keybinds.push(("d".to_string(), "Down split".to_string()));
            keybinds.push(("r".to_string(), "Right split".to_string()));
            keybinds.push(("x".to_string(), "Close".to_string()));
            keybinds.push(("s".to_string(), "Sync".to_string()));
            keybinds.push(("f".to_string(), "Fullscreen".to_string()));
        }
        InputMode::Tab => {
            keybinds.push(("←↓↑→".to_string(), "Move focus".to_string()));
            keybinds.push(("n".to_string(), "New".to_string()));
            keybinds.push(("x".to_string(), "Close".to_string()));
            keybinds.push(("r".to_string(), "Rename".to_string()));
            keybinds.push(("s".to_string(), "Search".to_string()));
        }
        InputMode::Scroll => {
            keybinds.push(("↓↑".to_string(), "Scroll".to_string()));
            keybinds.push(("PgUp/PgDn".to_string(), "Scroll Page".to_string()));
        }
        InputMode::RenameTab => {
            keybinds.push(("Enter".to_string(), "when done".to_string()));
        }
        InputMode::SearchTab => {
            keybinds.push(("Enter".to_string(), "when done".to_string()));
        }
    }
    ModeInfo {
        mode,
        keybinds,
        palette,
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
