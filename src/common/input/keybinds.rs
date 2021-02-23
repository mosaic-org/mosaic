//! Mapping of inputs to sequences of actions.

use super::actions::{Action, Direction};
use super::handler::InputMode;

use std::collections::HashMap;

use strum::IntoEnumIterator;
use termion::event::Key;

type Keybinds = HashMap<InputMode, ModeKeybinds>;
type ModeKeybinds = HashMap<Key, Vec<Action>>;

/// Populates the default hashmap of keybinds.
/// @@@khs26 What about an input config file?
pub fn get_default_keybinds() -> Result<Keybinds, String> {
    let mut defaults = Keybinds::new();

    for mode in InputMode::iter() {
        defaults.insert(mode, get_defaults_for_mode(&mode)?);
    }

    Ok(defaults)
}

/// Returns the default keybinds for a givent [`InputMode`].
fn get_defaults_for_mode(mode: &InputMode) -> Result<ModeKeybinds, String> {
    let mut defaults = ModeKeybinds::new();

    match *mode {
        InputMode::Normal => {
            defaults.insert(
                Key::Ctrl('g'),
                vec![Action::SwitchToMode(InputMode::Command)],
            );
        }
        InputMode::Command => {
            defaults.insert(
                Key::Char('r'),
                vec![Action::SwitchToMode(InputMode::Resize)],
            );
            defaults.insert(Key::Char('p'), vec![Action::SwitchToMode(InputMode::Pane)]);
            defaults.insert(Key::Char('t'), vec![Action::SwitchToMode(InputMode::Tab)]);
            defaults.insert(
                Key::Char('s'),
                vec![Action::SwitchToMode(InputMode::Scroll)],
            );
            defaults.insert(
                Key::Ctrl('g'),
                vec![Action::SwitchToMode(InputMode::Normal)],
            );
            defaults.insert(Key::Esc, vec![Action::SwitchToMode(InputMode::Normal)]);
            defaults.insert(Key::Char('q'), vec![Action::Quit]);
        }
        InputMode::Resize => {
            defaults.insert(Key::Char('h'), vec![Action::Resize(Direction::Left)]);
            defaults.insert(Key::Char('j'), vec![Action::Resize(Direction::Down)]);
            defaults.insert(Key::Char('k'), vec![Action::Resize(Direction::Up)]);
            defaults.insert(Key::Char('l'), vec![Action::Resize(Direction::Right)]);

            defaults.insert(Key::Left, vec![Action::Resize(Direction::Left)]);
            defaults.insert(Key::Down, vec![Action::Resize(Direction::Down)]);
            defaults.insert(Key::Up, vec![Action::Resize(Direction::Up)]);
            defaults.insert(Key::Right, vec![Action::Resize(Direction::Right)]);

            defaults.insert(Key::Ctrl('b'), vec![Action::Resize(Direction::Left)]);
            defaults.insert(Key::Ctrl('n'), vec![Action::Resize(Direction::Down)]);
            defaults.insert(Key::Ctrl('p'), vec![Action::Resize(Direction::Up)]);
            defaults.insert(Key::Ctrl('f'), vec![Action::Resize(Direction::Right)]);

            defaults.insert(Key::Char('q'), vec![Action::Quit]);
            defaults.insert(
                Key::Ctrl('g'),
                vec![Action::SwitchToMode(InputMode::Normal)],
            );
            defaults.insert(Key::Esc, vec![Action::SwitchToMode(InputMode::Command)]);
        }
        InputMode::Pane => {
            defaults.insert(Key::Char('h'), vec![Action::MoveFocus(Direction::Left)]);
            defaults.insert(Key::Char('j'), vec![Action::MoveFocus(Direction::Down)]);
            defaults.insert(Key::Char('k'), vec![Action::MoveFocus(Direction::Up)]);
            defaults.insert(Key::Char('l'), vec![Action::MoveFocus(Direction::Right)]);

            defaults.insert(Key::Left, vec![Action::MoveFocus(Direction::Left)]);
            defaults.insert(Key::Down, vec![Action::MoveFocus(Direction::Down)]);
            defaults.insert(Key::Up, vec![Action::MoveFocus(Direction::Up)]);
            defaults.insert(Key::Right, vec![Action::MoveFocus(Direction::Right)]);

            defaults.insert(Key::Ctrl('b'), vec![Action::MoveFocus(Direction::Left)]);
            defaults.insert(Key::Ctrl('n'), vec![Action::MoveFocus(Direction::Down)]);
            defaults.insert(Key::Ctrl('p'), vec![Action::MoveFocus(Direction::Up)]);
            defaults.insert(Key::Ctrl('f'), vec![Action::MoveFocus(Direction::Right)]);

            defaults.insert(Key::Char('p'), vec![Action::SwitchFocus(Direction::Right)]);
            defaults.insert(Key::Char('n'), vec![Action::NewPane(None)]);
            defaults.insert(Key::Char('d'), vec![Action::NewPane(Some(Direction::Down))]);
            defaults.insert(
                Key::Char('r'),
                vec![Action::NewPane(Some(Direction::Right))],
            );
            defaults.insert(Key::Char('x'), vec![Action::CloseFocus]);

            defaults.insert(Key::Char('f'), vec![Action::ToggleFocusFullscreen]);

            defaults.insert(Key::Char('q'), vec![Action::Quit]);
            defaults.insert(
                Key::Ctrl('g'),
                vec![Action::SwitchToMode(InputMode::Normal)],
            );
            defaults.insert(Key::Esc, vec![Action::SwitchToMode(InputMode::Command)]);
        }
        InputMode::Tab => {
            defaults.insert(Key::Char('h'), vec![Action::GoToPreviousTab]);
            defaults.insert(Key::Char('j'), vec![Action::GoToNextTab]);
            defaults.insert(Key::Char('k'), vec![Action::GoToPreviousTab]);
            defaults.insert(Key::Char('l'), vec![Action::GoToNextTab]);

            defaults.insert(Key::Left, vec![Action::GoToPreviousTab]);
            defaults.insert(Key::Down, vec![Action::GoToNextTab]);
            defaults.insert(Key::Up, vec![Action::GoToPreviousTab]);
            defaults.insert(Key::Right, vec![Action::GoToNextTab]);

            defaults.insert(Key::Ctrl('b'), vec![Action::GoToPreviousTab]);
            defaults.insert(Key::Ctrl('n'), vec![Action::GoToNextTab]);
            defaults.insert(Key::Ctrl('p'), vec![Action::GoToPreviousTab]);
            defaults.insert(Key::Ctrl('f'), vec![Action::GoToNextTab]);

            defaults.insert(Key::Char('n'), vec![Action::NewTab]);
            defaults.insert(Key::Char('x'), vec![Action::CloseTab]);

            defaults.insert(Key::Char('q'), vec![Action::Quit]);
            defaults.insert(
                Key::Ctrl('g'),
                vec![Action::SwitchToMode(InputMode::Normal)],
            );
            defaults.insert(Key::Esc, vec![Action::SwitchToMode(InputMode::Command)]);
        }
        InputMode::Scroll => {
            defaults.insert(Key::Char('j'), vec![Action::ScrollDown]);
            defaults.insert(Key::Char('k'), vec![Action::ScrollUp]);

            defaults.insert(Key::Down, vec![Action::ScrollDown]);
            defaults.insert(Key::Up, vec![Action::ScrollUp]);

            defaults.insert(Key::Ctrl('n'), vec![Action::ScrollDown]);
            defaults.insert(Key::Ctrl('p'), vec![Action::ScrollUp]);

            defaults.insert(Key::Char('q'), vec![Action::Quit]);
            defaults.insert(
                Key::Ctrl('g'),
                vec![Action::SwitchToMode(InputMode::Normal)],
            );
            defaults.insert(Key::Esc, vec![Action::SwitchToMode(InputMode::Command)]);
        }
    }

    Ok(defaults)
}

/// Converts a [`Key`] terminal event to a sequence of [`Action`]s according to the current
/// [`InputMode`] and [`Keybinds`].
pub fn key_to_actions(
    key: &Key,
    input: Vec<u8>,
    mode: &InputMode,
    keybinds: &Keybinds,
) -> Vec<Action> {
    let mode_keybind_or_action = |action: Action| {
        keybinds
            .get(mode)
            .unwrap_or_else(|| unreachable!("Unrecognized mode: {:?}", mode))
            .get(key)
            .cloned()
            .unwrap_or_else(|| vec![action])
    };
    match *mode {
        InputMode::Normal => mode_keybind_or_action(Action::Write(input)),
        _ => mode_keybind_or_action(Action::NoOp),
    }
}
