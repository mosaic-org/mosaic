//! Things related to [`Screen`]s.

use std::collections::BTreeMap;
use std::os::unix::io::RawFd;
use std::str;
use std::sync::mpsc::Receiver;

use super::{AppInstruction, SenderWithContext};
use crate::os_input_output::OsApi;
use crate::panes::PositionAndSize;
use crate::pty_bus::{PtyInstruction, VteBytes};
use crate::tab::Tab;
use crate::{errors::ErrorContext, wasm_vm::PluginInstruction};
use crate::{layout::Layout, panes::PaneId};

use zellij_tile::data::{Event, InputMode, ModeInfo, Palette, TabInfo};

/// Instructions that can be sent to the [`Screen`].
#[derive(Debug, Clone)]
pub enum ScreenInstruction {
    PtyBytes(RawFd, VteBytes),
    Render,
    NewPane(PaneId),
    HorizontalSplit(PaneId),
    VerticalSplit(PaneId),
    WriteCharacter(Vec<u8>),
    ResizeLeft,
    ResizeRight,
    ResizeDown,
    ResizeUp,
    SwitchFocus,
    FocusNextPane,
    FocusPreviousPane,
    MoveFocusLeft,
    MoveFocusDown,
    MoveFocusUp,
    MoveFocusRight,
    Quit,
    ScrollUp,
    ScrollDown,
    PageScrollUp,
    PageScrollDown,
    ClearScroll,
    CloseFocusedPane,
    ToggleActiveTerminalFullscreen,
    SetSelectable(PaneId, bool),
    SetMaxHeight(PaneId, usize),
    SetInvisibleBorders(PaneId, bool),
    ClosePane(PaneId),
    ApplyLayout((Layout, Vec<RawFd>)),
    NewTab(RawFd),
    SwitchTabNext,
    SwitchTabPrev,
    ToggleActiveSyncPanes,
    CloseTab,
    GoToTab(u32),
    UpdateTabName(Vec<u8>),
    TerminalResize,
    ChangeMode(ModeInfo),
}

/// A [`Screen`] holds multiple [`Tab`]s, each one holding multiple [`panes`](crate::client::panes).
/// It only directly controls which tab is active, delegating the rest to the individual `Tab`.
pub struct Screen {
    /// A [`ScreenInstruction`] and [`ErrorContext`] receiver.
    pub receiver: Receiver<(ScreenInstruction, ErrorContext)>,
    /// An optional maximal amount of panes allowed per [`Tab`] in this [`Screen`] instance.
    max_panes: Option<usize>,
    /// A map between this [`Screen`]'s tabs and their ID/key.
    tabs: BTreeMap<usize, Tab>,
    /// A [`PtyInstruction`] and [`ErrorContext`] sender.
    pub send_pty_instructions: SenderWithContext<PtyInstruction>,
    /// A [`PluginInstruction`] and [`ErrorContext`] sender.
    pub send_plugin_instructions: SenderWithContext<PluginInstruction>,
    /// An [`AppInstruction`] and [`ErrorContext`] sender.
    pub send_app_instructions: SenderWithContext<AppInstruction>,
    /// The full size of this [`Screen`].
    full_screen_ws: PositionAndSize,
    /// The index of this [`Screen`]'s active [`Tab`].
    active_tab_index: Option<usize>,
    /// The [`OsApi`] this [`Screen`] uses.
    os_api: Box<dyn OsApi>,
    mode_info: ModeInfo,
    input_mode: InputMode,
    colors: Palette,
}

impl Screen {
    // FIXME: This lint needs actual fixing! Maybe by bundling the Senders
    /// Creates and returns a new [`Screen`].
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        receive_screen_instructions: Receiver<(ScreenInstruction, ErrorContext)>,
        send_pty_instructions: SenderWithContext<PtyInstruction>,
        send_plugin_instructions: SenderWithContext<PluginInstruction>,
        send_app_instructions: SenderWithContext<AppInstruction>,
        full_screen_ws: &PositionAndSize,
        os_api: Box<dyn OsApi>,
        max_panes: Option<usize>,
        mode_info: ModeInfo,
        input_mode: InputMode,
        colors: Palette,
    ) -> Self {
        Screen {
            receiver: receive_screen_instructions,
            max_panes,
            send_pty_instructions,
            send_plugin_instructions,
            send_app_instructions,
            full_screen_ws: *full_screen_ws,
            active_tab_index: None,
            tabs: BTreeMap::new(),
            os_api,
            mode_info,
            input_mode,
            colors,
        }
    }

    /// Creates a new [`Tab`] in this [`Screen`], containing a single
    /// [pane](crate::client::panes) with PTY file descriptor `pane_id`.
    pub fn new_tab(&mut self, pane_id: RawFd) {
        let tab_index = self.get_new_tab_index();
        let position = self.tabs.len();
        let tab = Tab::new(
            tab_index,
            position,
            String::new(),
            &self.full_screen_ws,
            self.os_api.clone(),
            self.send_pty_instructions.clone(),
            self.send_plugin_instructions.clone(),
            self.send_app_instructions.clone(),
            self.max_panes,
            Some(PaneId::Terminal(pane_id)),
            self.mode_info.clone(),
            self.input_mode,
            self.colors,
        );
        self.active_tab_index = Some(tab_index);
        self.tabs.insert(tab_index, tab);
        self.update_tabs();
        self.render();
    }

    /// Returns the index where a new [`Tab`] should be created in this [`Screen`].
    /// Currently, this is right after the last currently existing tab, or `0` if
    /// no tabs exist in this screen yet.
    fn get_new_tab_index(&self) -> usize {
        if let Some(index) = self.tabs.keys().last() {
            *index + 1
        } else {
            0
        }
    }

    /// Sets this [`Screen`]'s active [`Tab`] to the next tab.
    pub fn switch_tab_next(&mut self) {
        let active_tab_pos = self.get_active_tab().unwrap().position;
        let new_tab_pos = (active_tab_pos + 1) % self.tabs.len();

        for tab in self.tabs.values_mut() {
            if tab.position == new_tab_pos {
                tab.set_force_render();
                self.active_tab_index = Some(tab.index);
                break;
            }
        }
        self.update_tabs();
        self.render();
    }

    /// Sets this [`Screen`]'s active [`Tab`] to the previous tab.
    pub fn switch_tab_prev(&mut self) {
        let active_tab_pos = self.get_active_tab().unwrap().position;
        let new_tab_pos = if active_tab_pos == 0 {
            self.tabs.len() - 1
        } else {
            active_tab_pos - 1
        };
        for tab in self.tabs.values_mut() {
            if tab.position == new_tab_pos {
                tab.set_force_render();
                self.active_tab_index = Some(tab.index);
                break;
            }
        }
        self.update_tabs();
        self.render();
    }

    pub fn go_to_tab(&mut self, mut tab_index: usize) {
        tab_index -= 1;
        let active_tab_index = self.get_active_tab().unwrap().index;
        if let Some(t) = self.tabs.values_mut().find(|t| t.position == tab_index) {
            if t.index != active_tab_index {
                t.set_force_render();
                self.active_tab_index = Some(t.index);
                self.update_tabs();
                self.render();
            }
        }
    }

    /// Closes this [`Screen`]'s active [`Tab`], exiting the application if it happens
    /// to be the last tab.
    pub fn close_tab(&mut self) {
        let active_tab_index = self.active_tab_index.unwrap();
        if self.tabs.len() > 1 {
            self.switch_tab_prev();
        }
        let active_tab = self.tabs.remove(&active_tab_index).unwrap();
        let pane_ids = active_tab.get_pane_ids();
        // below we don't check the result of sending the CloseTab instruction to the pty thread
        // because this might be happening when the app is closing, at which point the pty thread
        // has already closed and this would result in an error
        let _ = self
            .send_pty_instructions
            .send(PtyInstruction::CloseTab(pane_ids));
        if self.tabs.is_empty() {
            self.active_tab_index = None;
            self.send_app_instructions
                .send(AppInstruction::Exit)
                .unwrap();
        } else {
            for t in self.tabs.values_mut() {
                if t.position > active_tab.position {
                    t.position -= 1;
                }
            }
            self.update_tabs();
        }
    }

    pub fn resize_to_screen(&mut self) {
        let new_screen_size = self.os_api.get_terminal_size_using_fd(0);
        self.full_screen_ws = new_screen_size;
        for (_, tab) in self.tabs.iter_mut() {
            tab.resize_whole_tab(new_screen_size);
        }
        let _ = self.get_active_tab_mut().map(|t| t.set_force_render());
        self.render();
    }

    /// Renders this [`Screen`], which amounts to rendering its active [`Tab`].
    pub fn render(&mut self) {
        if let Some(active_tab) = self.get_active_tab_mut() {
            if active_tab.get_active_pane().is_some() {
                active_tab.render();
            } else {
                self.close_tab();
            }
        };
    }

    /// Returns a mutable reference to this [`Screen`]'s tabs.
    pub fn get_tabs_mut(&mut self) -> &mut BTreeMap<usize, Tab> {
        &mut self.tabs
    }

    /// Returns an immutable reference to this [`Screen`]'s active [`Tab`].
    pub fn get_active_tab(&self) -> Option<&Tab> {
        match self.active_tab_index {
            Some(tab) => self.tabs.get(&tab),
            None => None,
        }
    }

    /// Returns a mutable reference to this [`Screen`]'s active [`Tab`].
    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        match self.active_tab_index {
            Some(tab) => self.get_tabs_mut().get_mut(&tab),
            None => None,
        }
    }

    /// Creates a new [`Tab`] in this [`Screen`], applying the specified [`Layout`]
    /// and switching to it.
    pub fn apply_layout(&mut self, layout: Layout, new_pids: Vec<RawFd>) {
        let tab_index = self.get_new_tab_index();
        let position = self.tabs.len();
        let mut tab = Tab::new(
            tab_index,
            position,
            String::new(),
            &self.full_screen_ws,
            self.os_api.clone(),
            self.send_pty_instructions.clone(),
            self.send_plugin_instructions.clone(),
            self.send_app_instructions.clone(),
            self.max_panes,
            None,
            self.mode_info.clone(),
            self.input_mode,
            self.colors,
        );
        tab.apply_layout(layout, new_pids);
        self.active_tab_index = Some(tab_index);
        self.tabs.insert(tab_index, tab);
        self.update_tabs();
    }

    pub fn update_tabs(&self) {
        let mut tab_data = vec![];
        let active_tab_index = self.active_tab_index.unwrap();
        for tab in self.tabs.values() {
            tab_data.push(TabInfo {
                position: tab.position,
                name: tab.name.clone(),
                active: active_tab_index == tab.index,
                is_fullscreen_active: tab.is_fullscreen_active(),
                is_sync_panes_active: tab.is_sync_panes_active(),
            });
        }
        self.send_plugin_instructions
            .send(PluginInstruction::Update(None, Event::TabUpdate(tab_data)))
            .unwrap();
    }

    pub fn update_active_tab_name(&mut self, buf: Vec<u8>) {
        let s = str::from_utf8(&buf).unwrap();
        let active_tab = self.get_active_tab_mut().unwrap();
        match s {
            "\0" => {
                active_tab.name = String::new();
            }
            "\u{007F}" | "\u{0008}" => {
                //delete and backspace keys
                active_tab.name.pop();
            }
            c => {
                active_tab.name.push_str(c);
            }
        }
        self.update_tabs();
    }
    pub fn change_mode(&mut self, mode_info: ModeInfo) {
        self.mode_info = mode_info;
        for tab in self.tabs.values_mut() {
            tab.mode_info = self.mode_info.clone();
        }
    }
}
