use ::insta::assert_snapshot;
use ::nix::pty::Winsize;

use crate::tests::fakes::FakeInputOutput;
use crate::tests::utils::get_output_frame_snapshots;
use crate::{start, Opt};

use crate::tests::utils::commands::{
    CLOSE_FOCUSED_PANE, MOVE_FOCUS, QUIT, RESIZE_DOWN, RESIZE_LEFT, RESIZE_UP, SPLIT_HORIZONTALLY,
    SPLIT_VERTICALLY,
};

fn get_fake_os_input(fake_win_size: &Winsize) -> FakeInputOutput {
    FakeInputOutput::new(fake_win_size.clone())
}

#[test]
pub fn adding_new_terminal_in_fullscreen() {
    let fake_win_size = Winsize {
        ws_col: 121,
        ws_row: 20,
        ws_xpixel: 0,
        ws_ypixel: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);
    fake_input_output.add_terminal_input(&[
        SPLIT_VERTICALLY,
        TOGGLE_ACTIVE_TERMINAL_FULLSCREEN,
        SPLIT_HORIZONTALLY,
        CLOSE_FOCUSED_PANE,
        QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), Opt::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    for snapshot in snapshots {
        assert_snapshot!(snapshot);
    }
}

