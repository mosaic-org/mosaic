//! Main input logic.

use super::actions::Action;
use super::keybinds::Keybinds;
use crate::common::input::config::Config;
use crate::common::{update_state, AppInstruction, AppState, SenderWithContext, OPENCALLS};
use crate::errors::ContextType;
use crate::os_input_output::OsApi;
use crate::pty_bus::PtyInstruction;
use crate::screen::ScreenInstruction;
use crate::wasm_vm::{EventType, PluginInputType, PluginInstruction};
use crate::CommandIsExecuting;

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use termion::input::TermReadEventsAndRaw;

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
        }
    }

    /// Main input event loop. Interprets the terminal [`Event`](termion::event::Event)s
    /// as [`Action`]s according to the current [`InputMode`], and dispatches those actions.
    fn handle_input(&mut self) {
        let mut err_ctx = OPENCALLS.with(|ctx| *ctx.borrow());
        err_ctx.add_call(ContextType::StdinHandler);
        self.send_pty_instructions.update(err_ctx);
        self.send_app_instructions.update(err_ctx);
        self.send_screen_instructions.update(err_ctx);
        let keybinds = self.config.keybinds.clone();
        'input_loop: loop {
            //@@@ I think this should actually just iterate over stdin directly
            let stdin_buffer = self.os_input.read_from_stdin();
            drop(
                self.send_plugin_instructions
                    .send(PluginInstruction::GlobalInput(stdin_buffer.clone())),
            );
            for key_result in stdin_buffer.events_and_raw() {
                match key_result {
                    Ok((event, raw_bytes)) => match event {
                        termion::event::Event::Key(key) => {
                            // FIXME this explicit break is needed because the current test
                            // framework relies on it to not create dead threads that loop
                            // and eat up CPUs. Do not remove until the test framework has
                            // been revised. Sorry about this (@categorille)
                            if {
                                let mut should_break = false;
                                // Hacked on way to have a means of testing Macros, needs to
                                // get properly integrated
                                for action in
                                    Keybinds::key_to_actions(&key, raw_bytes, &self.mode, &keybinds)
                                {
                                    should_break |= self.dispatch_action(action);
                                }
                                should_break
                            } {
                                break 'input_loop;
                            }
                        }
                        termion::event::Event::Mouse(_) | termion::event::Event::Unsupported(_) => {
                            unimplemented!("Mouse and unsupported events aren't supported!");
                        }
                    },
                    Err(err) => panic!("Encountered read error: {:?}", err),
                }
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
                update_state(&self.send_app_instructions, |_| AppState {
                    input_mode: self.mode,
                });
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
            Action::SwitchFocus(_) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::MoveFocus)
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
                self.check_escape();
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
                self.check_escape();
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
            Action::CloseTab => {
                self.command_is_executing.closing_pane();
                self.send_screen_instructions
                    .send(ScreenInstruction::CloseTab)
                    .unwrap();
                self.command_is_executing.wait_until_pane_is_closed();
                self.check_escape();
            }
            Action::GoToTab(i) => {
                self.send_screen_instructions
                    .send(ScreenInstruction::GoToTab(i))
                    .unwrap();
                self.check_escape();
            }
            Action::TabNameInput(c) => {
                self.send_plugin_instructions
                    .send(PluginInstruction::Input(
                        PluginInputType::Event(EventType::Tab),
                        c.clone(),
                    ))
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::UpdateTabName(c))
                    .unwrap();
            }
            Action::SaveTabName => {
                self.send_plugin_instructions
                    .send(PluginInstruction::Input(
                        PluginInputType::Event(EventType::Tab),
                        vec![b'\n'],
                    ))
                    .unwrap();
                self.send_screen_instructions
                    .send(ScreenInstruction::UpdateTabName(vec![b'\n']))
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

    fn check_escape(&mut self) {
        if self.config.auto_escape {
            self.dispatch_action(Action::SwitchToMode(InputMode::Normal));
        }
    }
}

/// Describes the different input modes, which change the way that keystrokes will be interpreted.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone, EnumIter, Serialize, Deserialize)]
pub enum InputMode {
    /// In `Normal` mode, input is always written to the terminal, except for the shortcuts leading
    /// to other modes
    #[serde(alias = "normal")]
    Normal,
    /// In `Locked` mode, input is always written to the terminal and all shortcuts are disabled
    /// except the one leading back to normal mode
    #[serde(alias = "locked")]
    Locked,
    /// `Resize` mode allows resizing the different existing panes.
    #[serde(alias = "resize")]
    Resize,
    /// `Pane` mode allows creating and closing panes, as well as moving between them.
    #[serde(alias = "pane")]
    Pane,
    /// `Tab` mode allows creating and closing tabs, as well as moving between them.
    #[serde(alias = "tab")]
    Tab,
    /// `Scroll` mode allows scrolling up and down within a pane.
    #[serde(alias = "scroll")]
    Scroll,
    #[serde(alias = "renametab")]
    RenameTab,
}

/// Represents the contents of the help message that is printed in the status bar,
/// which indicates the current [`InputMode`] and what the keybinds for that mode
/// are. Related to the default `status-bar` plugin.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Help {
    pub mode: InputMode,
    pub keybinds: Vec<(String, String)>, // <shortcut> => <shortcut description>
}

impl Default for InputMode {
    fn default() -> InputMode {
        InputMode::Normal
    }
}

/// Creates a [`Help`] struct indicating the current [`InputMode`] and its keybinds
/// (as pairs of [`String`]s).
// TODO this should probably be automatically generated in some way
pub fn get_help(mode: InputMode) -> Help {
    let mut keybinds: Vec<(String, String)> = vec![];
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
            keybinds.push(("f".to_string(), "Fullscreen".to_string()));
        }
        InputMode::Tab => {
            keybinds.push(("←↓↑→".to_string(), "Move focus".to_string()));
            keybinds.push(("n".to_string(), "New".to_string()));
            keybinds.push(("x".to_string(), "Close".to_string()));
            keybinds.push(("r".to_string(), "Rename".to_string()));
        }
        InputMode::Scroll => {
            keybinds.push(("↓↑".to_string(), "Scroll".to_string()));
        }
        InputMode::RenameTab => {
            keybinds.push(("Enter".to_string(), "when done".to_string()));
        }
    }
    Help { mode, keybinds }
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
