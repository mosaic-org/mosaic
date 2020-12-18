use std::collections::BTreeMap;
use std::os::unix::io::RawFd;
use std::sync::mpsc::Receiver;

use crate::errors::ErrorContext;
use crate::layout::Layout;
use crate::os_input_output::OsApi;
use crate::pty_bus::{PtyInstruction, VteEvent};
use crate::tab::Tab;
use crate::terminal_pane::PositionAndSize;
use crate::{AppInstruction, SenderWithContext};

/*
 * Screen
 *
 * this holds multiple panes (currently terminal panes) which are currently displayed on screen
 * it tracks their coordinates (x/y) and size, as well as how they should be resized
 *
 */

#[derive(Debug, Clone)]
pub enum ScreenInstruction {
    Pty(RawFd, VteEvent),
    Render,
    NewPane(RawFd),
    HorizontalSplit(RawFd),
    VerticalSplit(RawFd),
    WriteCharacter(Vec<u8>),
    ResizeLeft,
    ResizeRight,
    ResizeDown,
    ResizeUp,
    MoveFocus,
    MoveFocusLeft,
    MoveFocusDown,
    MoveFocusUp,
    MoveFocusRight,
    Quit,
    ScrollUp,
    ScrollDown,
    ClearScroll,
    CloseFocusedPane,
    ToggleActiveTerminalFullscreen,
    ClosePane(RawFd),
    ApplyLayout((Layout, Vec<RawFd>)),
    NewTab(RawFd),
    SwitchTabNext,
    SwitchTabPrev,
    CloseTab,
}

pub struct Screen {
    pub receiver: Receiver<(ScreenInstruction, ErrorContext)>,
    max_panes: Option<usize>,
    tabs: BTreeMap<usize, Tab>,
    pub send_pty_instructions: SenderWithContext<PtyInstruction>,
    pub send_app_instructions: SenderWithContext<AppInstruction>,
    full_screen_ws: PositionAndSize,
    active_tab: Option<usize>,
    os_api: Box<dyn OsApi>,
    next_tab_index: usize,
}

impl Screen {
    pub fn new(
        receive_screen_instructions: Receiver<(ScreenInstruction, ErrorContext)>,
        send_pty_instructions: SenderWithContext<PtyInstruction>,
        send_app_instructions: SenderWithContext<AppInstruction>,
        full_screen_ws: &PositionAndSize,
        os_api: Box<dyn OsApi>,
        max_panes: Option<usize>,
    ) -> Self {
        Screen {
            receiver: receive_screen_instructions,
            max_panes,
            send_pty_instructions,
            send_app_instructions,
            full_screen_ws: *full_screen_ws,
            active_tab: None,
            tabs: BTreeMap::new(),
            os_api,
            next_tab_index: 0,
        }
    }
    pub fn new_tab(&mut self, pane_id: RawFd) {
        let tab = Tab::new(
            self.next_tab_index,
            &self.full_screen_ws,
            self.os_api.clone(),
            self.send_pty_instructions.clone(),
            self.send_app_instructions.clone(),
            self.max_panes,
            Some(pane_id),
        );
        self.active_tab = Some(tab.index);
        self.tabs.insert(self.next_tab_index, tab);
        self.next_tab_index += 1;
        self.render();
    }
    pub fn switch_tab_next(&mut self) {
        let active_tab_id = self.get_active_tab().unwrap().index;
        let tab_ids: Vec<usize> = self.tabs.keys().copied().collect();
        let first_tab = tab_ids.get(0).unwrap();
        let active_tab_id_position = tab_ids.iter().position(|id| id == &active_tab_id).unwrap();
        if let Some(next_tab) = tab_ids.get(active_tab_id_position + 1) {
            self.active_tab = Some(*next_tab);
        } else {
            self.active_tab = Some(*first_tab);
        }
        self.render();
    }
    pub fn switch_tab_prev(&mut self) {
        let active_tab_id = self.get_active_tab().unwrap().index;
        let tab_ids: Vec<usize> = self.tabs.keys().copied().collect();
        let first_tab = tab_ids.get(0).unwrap();
        let last_tab = tab_ids.get(tab_ids.len() - 1).unwrap();

        let active_tab_id_position = tab_ids.iter().position(|id| id == &active_tab_id).unwrap();
        if active_tab_id == *first_tab {
            self.active_tab = Some(*last_tab)
        } else if let Some(prev_tab) = tab_ids.get(active_tab_id_position - 1) {
            self.active_tab = Some(*prev_tab)
        }
        self.render();
    }
    pub fn close_tab(&mut self) {
        let active_tab_index = self.active_tab.unwrap();
        if self.tabs.len() > 1 {
            self.switch_tab_prev();
        }
        let mut active_tab = self.tabs.remove(&active_tab_index).unwrap();
        let pane_ids = active_tab.get_pane_ids();
        self.send_pty_instructions
            .send(PtyInstruction::CloseTab(pane_ids))
            .unwrap();
        if self.tabs.len() == 0 {
            self.active_tab = None;
            self.render();
        }
    }
    pub fn render(&mut self) {
        let close_tab = if let Some(active_tab) = self.get_active_tab_mut() {
            if active_tab.get_active_terminal().is_some() {
                active_tab.render();
                false
            } else {
                true
            }
        } else {
            self.send_app_instructions
                .send(AppInstruction::Exit)
                .unwrap();
            false
        };
        if close_tab {
            self.close_tab();
        }
    }

    pub fn get_active_tab(&self) -> Option<&Tab> {
        match self.active_tab {
            Some(tab) => self.tabs.get(&tab),
            None => None,
        }
    }
    pub fn get_tabs_mut(&mut self) -> &mut BTreeMap<usize, Tab> {
        &mut self.tabs
    }
    pub fn get_active_tab_mut(&mut self) -> Option<&mut Tab> {
        let tab = match self.active_tab {
            Some(tab) => self.get_tabs_mut().get_mut(&tab),
            None => None,
        };
        tab
    }
    pub fn apply_layout(&mut self, layout: Layout, new_pids: Vec<RawFd>) {
        let mut tab = Tab::new(
            self.next_tab_index,
            &self.full_screen_ws,
            self.os_api.clone(),
            self.send_pty_instructions.clone(),
            self.send_app_instructions.clone(),
            self.max_panes,
            None,
        );
        tab.apply_layout(layout, new_pids);
        self.active_tab = Some(tab.index);
        self.tabs.insert(self.next_tab_index, tab);
        self.next_tab_index += 1;
    }
}
