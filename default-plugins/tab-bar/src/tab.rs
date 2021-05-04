use crate::{LinePart, ARROW_SEPARATOR};
use ansi_term::ANSIStrings;
use zellij_tile::prelude::*;
use zellij_tile_extra::*;

pub fn active_tab(text: String, palette: Palette) -> LinePart {
    let left_separator = style!(palette.bg, palette.green).paint(ARROW_SEPARATOR);
    let tab_text_len = text.chars().count() + 4; // 2 for left and right separators, 2 for the text padding
    let tab_styled_text = style!(palette.black, palette.green)
        .bold()
        .paint(format!(" {} ", text));
    let right_separator = style!(palette.green, palette.bg).paint(ARROW_SEPARATOR);
    let tab_styled_text = format!(
        "{}",
        ANSIStrings(&[left_separator, tab_styled_text, right_separator,])
    );
    LinePart {
        part: tab_styled_text,
        len: tab_text_len,
    }
}

pub fn non_active_tab(text: String, palette: Palette) -> LinePart {
    let left_separator = style!(palette.bg, palette.fg).paint(ARROW_SEPARATOR);
    let tab_text_len = text.chars().count() + 4; // 2 for left and right separators, 2 for the padding
    let tab_styled_text = style!(palette.black, palette.fg)
        .bold()
        .paint(format!(" {} ", text));
    let right_separator = style!(palette.fg, palette.bg).paint(ARROW_SEPARATOR);
    let tab_styled_text = format!(
        "{}",
        ANSIStrings(&[left_separator, tab_styled_text, right_separator,])
    );
    LinePart {
        part: tab_styled_text,
        len: tab_text_len,
    }
}

pub fn tab_style(
    text: String,
    is_active_tab: bool,
    position: usize,
    is_sync_panes_active: bool,
    palette: Palette,
) -> LinePart {
    let mut tab_text = if text.is_empty() {
        format!("Tab #{}", position + 1)
    } else {
        text
    };
    if is_sync_panes_active {
        tab_text.push_str(" (Sync)");
    }
    if is_active_tab {
        active_tab(tab_text, palette)
    } else {
        non_active_tab(tab_text, palette)
    }
}
