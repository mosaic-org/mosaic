//! Definition of the actions that can be bound to keys.

use super::handler;

/// The four directions (left, right, up, down).
#[derive(Clone)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// Actions that can be bound to keys.
#[derive(Clone)]
pub enum Action {
    /// Quit Zellij.
    Quit,
    /// Write to the terminal.
    Write(Vec<u8>),
    /// Switch to the specified input mode.
    SwitchToMode(handler::InputMode),
    /// Resize focus pane in specified direction.
    Resize(Direction),
    /// Switch focus to next pane in specified direction.
    SwitchFocus(Direction),
    /// Move the focus pane in specified direction.
    MoveFocus(Direction),
    /// Scroll up in focus pane.
    ScrollUp,
    /// Scroll down in focus pane.
    ScrollDown,
    /// Toggle between fullscreen focus pane and normal layout.
    ToggleFocusFullscreen,
    /// Open a new pane in the specified direction (relative to focus).
    /// If no direction is specified, will try to use the biggest available space.
    NewPane(Option<Direction>),
    /// Close the focus pane.
    CloseFocus,
    /// Create a new tab.
    NewTab,
    /// Do nothing.
    NoOp,
    /// Go to the next tab.
    GoToNextTab,
    /// Go to the previous tab.
    GoToPreviousTab,
    /// Close the current tab.
    CloseTab,
}
