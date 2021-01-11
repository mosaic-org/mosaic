#![allow(clippy::clippy::if_same_then_else)]

use ::nix::pty::Winsize;
use ::std::os::unix::io::RawFd;
use ::vte::Perform;
use std::fmt::{self, Debug, Formatter};

use crate::boundaries::Rect;
use crate::terminal_pane::grid::{Grid, Row};
use crate::terminal_pane::terminal_character::{
    AnsiCode, CharacterStyles, NamedColor, TerminalCharacter,
    EMPTY_TERMINAL_CHARACTER
};
use crate::utils::logging::debug_log_to_file;
use crate::VteEvent;

#[derive(Clone, Copy, Debug)]
pub struct PositionAndSize {
    pub x: usize,
    pub y: usize,
    pub rows: usize,
    pub columns: usize,
}

impl PositionAndSize {
    pub fn from(winsize: Winsize) -> PositionAndSize {
        PositionAndSize {
            columns: winsize.ws_col as usize,
            rows: winsize.ws_row as usize,
            x: winsize.ws_xpixel as usize,
            y: winsize.ws_ypixel as usize,
        }
    }
}

#[derive(Debug)]
pub struct TerminalPane {
    pub grid: Grid,
    pub alternative_grid: Option<Grid>, // for 1049h/l instructions which tell us to switch between these two
    pub pid: RawFd,
    pub should_render: bool,
    pub position_and_size: PositionAndSize,
    pub position_and_size_override: Option<PositionAndSize>,
    pub cursor_key_mode: bool, // DECCKM - when set, cursor keys should send ANSI direction codes (eg. "OD") instead of the arrow keys (eg. "[D")
    pending_styles: CharacterStyles,
}

impl Rect for TerminalPane {
    fn x(&self) -> usize {
        self.get_x()
    }
    fn y(&self) -> usize {
        self.get_y()
    }
    fn rows(&self) -> usize {
        self.get_rows()
    }
    fn columns(&self) -> usize {
        self.get_columns()
    }
}

impl Rect for &mut TerminalPane {
    fn x(&self) -> usize {
        self.get_x()
    }
    fn y(&self) -> usize {
        self.get_y()
    }
    fn rows(&self) -> usize {
        self.get_rows()
    }
    fn columns(&self) -> usize {
        self.get_columns()
    }
}

impl TerminalPane {
    pub fn new(pid: RawFd, ws: PositionAndSize, x: usize, y: usize) -> TerminalPane {
        let grid = Grid::new(ws.rows, ws.columns, pid);
        let pending_styles = CharacterStyles::new();
        let position_and_size = PositionAndSize {
            x,
            y,
            rows: ws.rows,
            columns: ws.columns,
        };
        TerminalPane {
            pid,
            grid,
            alternative_grid: None,
            should_render: true,
            pending_styles,
            position_and_size,
            position_and_size_override: None,
            cursor_key_mode: false,
        }
    }
    pub fn mark_for_rerender(&mut self) {
        self.should_render = true;
    }
    pub fn handle_event(&mut self, event: VteEvent) {
        match event {
            VteEvent::Print(c) => {
                self.print(c);
                self.mark_for_rerender();
            }
            VteEvent::Execute(byte) => {
                self.execute(byte);
            }
            VteEvent::Hook(params, intermediates, ignore, c) => {
                self.hook(&params, &intermediates, ignore, c);
            }
            VteEvent::Put(byte) => {
                self.put(byte);
            }
            VteEvent::Unhook => {
                self.unhook();
            }
            VteEvent::OscDispatch(params, bell_terminated) => {
                let params: Vec<&[u8]> = params.iter().map(|p| &p[..]).collect();
                self.osc_dispatch(&params[..], bell_terminated);
            }
            VteEvent::CsiDispatch(params, intermediates, ignore, c) => {
                self.csi_dispatch(&params, &intermediates, ignore, c);
            }
            VteEvent::EscDispatch(intermediates, ignore, byte) => {
                self.esc_dispatch(&intermediates, ignore, byte);
            }
        }
    }
    pub fn reduce_width_right(&mut self, count: usize) {
        self.position_and_size.x += count;
        self.position_and_size.columns -= count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn reduce_width_left(&mut self, count: usize) {
        self.position_and_size.columns -= count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn increase_width_left(&mut self, count: usize) {
        self.position_and_size.x -= count;
        self.position_and_size.columns += count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn increase_width_right(&mut self, count: usize) {
        self.position_and_size.columns += count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn reduce_height_down(&mut self, count: usize) {
        self.position_and_size.y += count;
        self.position_and_size.rows -= count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn increase_height_down(&mut self, count: usize) {
        self.position_and_size.rows += count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn increase_height_up(&mut self, count: usize) {
        self.position_and_size.y -= count;
        self.position_and_size.rows += count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn reduce_height_up(&mut self, count: usize) {
        self.position_and_size.rows -= count;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn change_size_p(&mut self, position_and_size: &PositionAndSize) {
        self.position_and_size = *position_and_size;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    // TODO: merge these two methods
    pub fn change_size(&mut self, ws: &PositionAndSize) {
        self.position_and_size.columns = ws.columns;
        self.position_and_size.rows = ws.rows;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn get_x(&self) -> usize {
        match self.position_and_size_override {
            Some(position_and_size_override) => position_and_size_override.x,
            None => self.position_and_size.x as usize,
        }
    }
    pub fn get_y(&self) -> usize {
        match self.position_and_size_override {
            Some(position_and_size_override) => position_and_size_override.y,
            None => self.position_and_size.y as usize,
        }
    }
    pub fn get_columns(&self) -> usize {
        match &self.position_and_size_override.as_ref() {
            Some(position_and_size_override) => position_and_size_override.columns,
            None => self.position_and_size.columns as usize,
        }
    }
    pub fn get_rows(&self) -> usize {
        match &self.position_and_size_override.as_ref() {
            Some(position_and_size_override) => position_and_size_override.rows,
            None => self.position_and_size.rows as usize,
        }
    }
    fn reflow_lines(&mut self) {
        let rows = self.get_rows();
        let columns = self.get_columns();
        self.grid.change_size(rows, columns);
    }
    pub fn buffer_as_vte_output(&mut self) -> Option<String> {
        // TODO: rename to render
        // if self.should_render {
        if true {
            // while checking should_render rather than rendering each pane every time
            // is more performant, it causes some problems when the pane to the left should be
            // rendered and has wide characters (eg. Chinese characters or emoji)
            // as a (hopefully) temporary hack, we render all panes until we find a better solution
            let mut vte_output = String::new();
            let buffer_lines = &self.read_buffer_as_lines();
            let display_cols = self.get_columns();
            let mut character_styles = CharacterStyles::new();
            for (row, line) in buffer_lines.iter().enumerate() {
                let x = self.get_x();
                let y = self.get_y();
                vte_output = format!("{}\u{1b}[{};{}H\u{1b}[m", vte_output, y + row + 1, x + 1); // goto row/col and reset styles
                for (col, t_character) in line.iter().enumerate() {
                    if col < display_cols {
                        // in some cases (eg. while resizing) some characters will spill over
                        // before they are corrected by the shell (for the prompt) or by reflowing
                        // lines
                        if let Some(new_styles) =
                            character_styles.update_and_return_diff(&t_character.styles)
                        {
                            // the terminal keeps the previous styles as long as we're in the same
                            // line, so we only want to update the new styles here (this also
                            // includes resetting previous styles as needed)
                            vte_output = format!("{}{}", vte_output, new_styles);
                        }
                        vte_output.push(t_character.character);
                    }
                }
                character_styles.clear();
            }
            self.mark_for_rerender();
            Some(vte_output)
        } else {
            None
        }
    }
    pub fn read_buffer_as_lines(&self) -> Vec<Vec<TerminalCharacter>> {
        self.grid.as_character_lines()
    }
    pub fn cursor_coordinates(&self) -> Option<(usize, usize)> {
        // (x, y)
        self.grid.cursor_coordinates()
    }
    pub fn scroll_up(&mut self, count: usize) {
        self.grid.move_viewport_up(count);
        self.mark_for_rerender();
    }
    pub fn scroll_down(&mut self, count: usize) {
        self.grid.move_viewport_down(count);
        self.mark_for_rerender();
    }
    pub fn rotate_scroll_region_up(&mut self, count: usize) {
        self.grid.rotate_scroll_region_up(count);
        self.mark_for_rerender();
    }
    pub fn rotate_scroll_region_down(&mut self, count: usize) {
        self.grid.rotate_scroll_region_down(count);
        self.mark_for_rerender();
    }
    pub fn clear_scroll(&mut self) {
        self.grid.reset_viewport();
        self.mark_for_rerender();
    }
    pub fn override_size_and_position(&mut self, x: usize, y: usize, size: &PositionAndSize) {
        let position_and_size_override = PositionAndSize {
            x,
            y,
            rows: size.rows,
            columns: size.columns,
        };
        self.position_and_size_override = Some(position_and_size_override);
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn reset_size_and_position_override(&mut self) {
        self.position_and_size_override = None;
        self.reflow_lines();
        self.mark_for_rerender();
    }
    pub fn adjust_input_to_terminal(&self, input_bytes: Vec<u8>) -> Vec<u8> {
        // there are some cases in which the terminal state means that input sent to it
        // needs to be adjusted.
        // here we match against those cases - if need be, we adjust the input and if not
        // we send back the original input
        match input_bytes.as_slice() {
            [27, 91, 68] => {
                // left arrow
                if self.cursor_key_mode {
                    // please note that in the line below, there is an ANSI escape code (27) at the beginning of the string,
                    // some editors will not show this
                    return "OD".as_bytes().to_vec();
                }
            }
            [27, 91, 67] => {
                // right arrow
                if self.cursor_key_mode {
                    // please note that in the line below, there is an ANSI escape code (27) at the beginning of the string,
                    // some editors will not show this
                    return "OC".as_bytes().to_vec();
                }
            }
            [27, 91, 65] => {
                // up arrow
                if self.cursor_key_mode {
                    // please note that in the line below, there is an ANSI escape code (27) at the beginning of the string,
                    // some editors will not show this
                    return "OA".as_bytes().to_vec();
                }
            }
            [27, 91, 66] => {
                // down arrow
                if self.cursor_key_mode {
                    // please note that in the line below, there is an ANSI escape code (27) at the beginning of the string,
                    // some editors will not show this
                    return "OB".as_bytes().to_vec();
                }
            }
            _ => {}
        };
        input_bytes
    }
    fn add_newline(&mut self) {
        self.grid.add_canonical_line();
        // self.reset_all_ansi_codes(); // TODO: find out if we should be resetting here or not
        self.mark_for_rerender();
    }
    fn move_to_beginning_of_line(&mut self) {
        self.grid.move_cursor_to_beginning_of_line();
    }
    fn move_cursor_backwards(&mut self, count: usize) {
        self.grid.move_cursor_backwards(count);
    }
    fn _reset_all_ansi_codes(&mut self) {
        self.pending_styles.clear();
    }
}

impl vte::Perform for TerminalPane {
    fn print(&mut self, c: char) {
        // apparently, building TerminalCharacter like this without a "new" method
        // is a little faster
        let terminal_character = TerminalCharacter {
            character: c,
            styles: self.pending_styles,
        };
        self.grid.add_character(terminal_character);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            8 => {
                // backspace
                self.move_cursor_backwards(1);
            }
            9 => {
                // tab
                let terminal_tab_character = TerminalCharacter {
                    character: '\t',
                    styles: self.pending_styles,
                };
                // TODO: handle better with line wrapping
                self.grid.add_character(terminal_tab_character);
            }
            10 => {
                // 0a, newline
                self.add_newline();
            }
            13 => {
                // 0d, carriage return
                self.move_to_beginning_of_line();
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _c: char) {
        // TBD
    }

    fn put(&mut self, _byte: u8) {
        // TBD
    }

    fn unhook(&mut self) {
        // TBD
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // TBD
    }

    fn csi_dispatch(&mut self, params: &[i64], _intermediates: &[u8], _ignore: bool, c: char) {
        if c == 'm' {
            self.pending_styles.add_style_from_ansi_params(params);
        } else if c == 'C' {
            // move cursor forward
            let move_by = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid.move_cursor_forward_until_edge(move_by);
        } else if c == 'K' {
            // clear line (0 => right, 1 => left, 2 => all)
            if params[0] == 0 {
                let mut char_to_replace = EMPTY_TERMINAL_CHARACTER;
                char_to_replace.styles = self.pending_styles;
                self.grid
                    .replace_characters_in_line_after_cursor(char_to_replace);
            } else if params[0] == 1 {
                let mut char_to_replace = EMPTY_TERMINAL_CHARACTER;
                char_to_replace.styles = self.pending_styles;
                self.grid
                    .replace_characters_in_line_before_cursor(char_to_replace);
            } else if params[0] == 2 {
                self.grid.clear_cursor_line();
            }
        } else if c == 'J' {
            // clear all (0 => below, 1 => above, 2 => all, 3 => saved)
            if params[0] == 0 {
                self.grid.clear_all_after_cursor();
            } else if params[0] == 2 {
                self.grid.clear_all();
            }
        // TODO: implement 1
        } else if c == 'H' {
            // goto row/col
            // we subtract 1 from the row/column because these are 1 indexed
            // (except when they are 0, in which case they should be 1
            // don't look at me, I don't make the rules)
            let (row, col) = if params.len() == 1 {
                if params[0] == 0 {
                    (0, params[0] as usize)
                } else {
                    (params[0] as usize - 1, params[0] as usize)
                }
            } else {
                if params[0] == 0 {
                    (0, params[1] as usize - 1)
                } else {
                    (params[0] as usize - 1, params[1] as usize - 1)
                }
            };
            self.grid.move_cursor_to(col, row);
        } else if c == 'A' {
            // move cursor up until edge of screen
            let move_up_count = if params[0] == 0 { 1 } else { params[0] };
            self.grid.move_cursor_up(move_up_count as usize);
        } else if c == 'B' {
            // move cursor down until edge of screen
            let move_down_count = if params[0] == 0 { 1 } else { params[0] };
            self.grid.move_cursor_down(move_down_count as usize);
        } else if c == 'D' {
            let move_back_count = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid.move_cursor_back(move_back_count);
        } else if c == 'l' {
            let first_intermediate_is_questionmark = match _intermediates.get(0) {
                Some(b'?') => true,
                None => false,
                _ => false,
            };
            if first_intermediate_is_questionmark {
                match params.get(0) {
                    Some(&1049) => {
                        if let Some(alternative_grid) = self.alternative_grid.as_mut() {
                            std::mem::swap(&mut self.grid, alternative_grid);
                            // self.grid = alternative_grid;
                        }
                        self.alternative_grid = None;
                    }
                    Some(&25) => {
                        self.grid.hide_cursor();
                        self.mark_for_rerender();
                    }
                    Some(&1) => {
                        self.cursor_key_mode = false;
                    }
                    _ => {}
                };
            }
        } else if c == 'h' {
            let first_intermediate_is_questionmark = match _intermediates.get(0) {
                Some(b'?') => true,
                None => false,
                _ => false,
            };
            if first_intermediate_is_questionmark {
                match params.get(0) {
                    Some(&25) => {
                        self.grid.show_cursor();
                        self.mark_for_rerender();
                    }
                    Some(&1049) => {
                        let columns = self.position_and_size_override.map(|x| x.columns).unwrap_or(self.position_and_size.columns);
                        let rows = self.position_and_size_override.map(|x| x.rows).unwrap_or(self.position_and_size.rows);
                        let current_grid = std::mem::replace(&mut self.grid, Grid::new(rows, columns, self.pid));
                        self.alternative_grid = Some(current_grid);
                    }
                    Some(&1) => {
                        self.cursor_key_mode = true;
                    }
                    _ => {}
                };
            }
        } else if c == 'r' {
            if params.len() > 1 {
                // minus 1 because these are 1 indexed
                let top_line_index = params[0] as usize - 1;
                let bottom_line_index = params[1] as usize - 1;
                self.grid
                    .set_scroll_region(top_line_index, bottom_line_index);
                self.grid.show_cursor();
            } else {
                self.grid.clear_scroll_region();
            }
        } else if c == 't' {
            // TBD - title?
        } else if c == 'n' {
            // TBD - device status report
        } else if c == 'c' {
            // TBD - identify terminal
        } else if c == 'M' {
            // delete lines if currently inside scroll region
            let line_count_to_delete = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid
                .delete_lines_in_scroll_region(line_count_to_delete);
        } else if c == 'L' {
            // insert blank lines if inside scroll region
            let line_count_to_add = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid
                .add_empty_lines_in_scroll_region(line_count_to_add);
        } else if c == 'q' {
            // ignore for now to run on mac
        } else if c == 'G' {
            let column = if params[0] == 0 {
                0
            } else {
                params[0] as usize - 1
            };
            self.grid.move_cursor_to_column(column);
        } else if c == 'd' {
            // goto line
            let line = if params[0] == 0 {
                1
            } else {
                // minus 1 because this is 1 indexed
                params[0] as usize - 1
            };
            self.grid.move_cursor_to_line(line);
        } else if c == 'P' {
            // erase characters
            let count = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid.erase_characters(count, self.pending_styles);
        } else if c == 'X' {
            // erase characters and replace with empty characters of current style
            let count = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid
                .replace_with_empty_chars(count, self.pending_styles);
        } else if c == 'T' {
            /*
             * 124  54  T   SD
             * Scroll down, new lines inserted at top of screen
             * [4T = Scroll down 4, bring previous lines back into view
             */
            let line_count: i64 = *params.get(0).expect("A number of lines was expected.");

            if line_count >= 0 {
                self.rotate_scroll_region_up(line_count as usize);
            } else {
                self.rotate_scroll_region_down(line_count.abs() as usize);
            }
        } else if c == 'S' {
            // move scroll up
            let count = if params[0] == 0 {
                1
            } else {
                params[0] as usize
            };
            self.grid.delete_lines_in_scroll_region(count);
            self.grid.add_empty_lines_in_scroll_region(count);
        } else {
            let _ = debug_log_to_file(format!("Unhandled csi: {}->{:?}", c, params));
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates.get(0)) {
            (b'M', None) => {
                self.grid.move_cursor_up_with_scrolling(1);
            }
            _ => {}
        }
    }
}
