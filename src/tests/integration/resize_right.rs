use ::insta::assert_snapshot;

use crate::panes::PositionAndSize;
use crate::tests::fakes::FakeInputOutput;
use crate::tests::utils::get_output_frame_snapshots;
use crate::{start, CliArgs};

use crate::tests::utils::commands::{
    COMMAND_TOGGLE, MOVE_FOCUS, QUIT, RESIZE_RIGHT, RESIZE_UP, SPLIT_HORIZONTALLY, SPLIT_VERTICALLY,
};

fn get_fake_os_input(fake_win_size: &PositionAndSize) -> FakeInputOutput {
    FakeInputOutput::new(*fake_win_size)
}

#[test]
pub fn resize_right_with_pane_to_the_left() {
    // ┌─────┬─────┐                   ┌───────┬───┐
    // │     │█████│                   │       │███│
    // │     │█████│ ==resize=right==> │       │███│
    // │     │█████│                   │       │███│
    // └─────┴─────┘                   └───────┴───┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);
    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &RESIZE_RIGHT,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_pane_to_the_right() {
    // ┌─────┬─────┐                   ┌───────┬───┐
    // │█████│     │                   │███████│   │
    // │█████│     │ ==resize=right==> │███████│   │
    // │█████│     │                   │███████│   │
    // └─────┴─────┘                   └───────┴───┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);
    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_left_and_right() {
    // ┌─────┬─────┬─────┐                   ┌─────┬───────┬───┐
    // │     │█████│     │                   │     │███████│   │
    // │     │█████│     │ ==resize=right==> │     │███████│   │
    // │     │█████│     │                   │     │███████│   │
    // └─────┴─────┴─────┘                   └─────┴───────┴───┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);
    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_multiple_panes_to_the_left() {
    // ┌─────┬─────┐                   ┌───────┬───┐
    // │     │█████│                   │       │███│
    // ├─────┤█████│ ==resize=right==> ├───────┤███│
    // │     │█████│                   │       │███│
    // └─────┴─────┘                   └───────┴───┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_left_aligned_top_with_current_pane() {
    // ┌─────┬─────┐                   ┌─────┬─────┐
    // │     │     │                   │     │     │
    // ├─────┼─────┤ ==resize=right==> ├─────┴─┬───┤
    // │     │█████│                   │       │███│
    // └─────┴─────┘                   └───────┴───┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_right_aligned_top_with_current_pane() {
    // ┌─────┬─────┐                   ┌─────┬─────┐
    // │     │     │                   │     │     │
    // ├─────┼─────┤ ==resize=right==> ├─────┴─┬───┤
    // │█████│     │                   │███████│   │
    // └─────┴─────┘                   └───────┴───┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &SPLIT_HORIZONTALLY,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_left_aligned_bottom_with_current_pane() {
    // ┌─────┬─────┐                   ┌───────┬───┐
    // │     │█████│                   │       │███│
    // ├─────┼─────┤ ==resize=right==> ├─────┬─┴───┤
    // │     │     │                   │     │     │
    // └─────┴─────┘                   └─────┴─────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_right_aligned_bottom_with_current_pane() {
    // ┌─────┬─────┐                   ┌───────┬───┐
    // │█████│     │                   │███████│   │
    // ├─────┼─────┤ ==resize=right==> ├─────┬─┴───┤
    // │     │     │                   │     │     │
    // └─────┴─────┘                   └─────┴─────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_VERTICALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_left_aligned_top_and_bottom_with_current_pane() {
    // ┌─────┬─────┐                   ┌─────┬─────┐
    // │     │     │                   │     │     │
    // ├─────┼─────┤                   ├─────┴─┬───┤
    // │     │█████│ ==resize=right==> │       │███│
    // ├─────┼─────┤                   ├─────┬─┴───┤
    // │     │     │                   │     │     │
    // └─────┴─────┘                   └─────┴─────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_HORIZONTALLY,
        &SPLIT_HORIZONTALLY,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_right_aligned_top_and_bottom_with_current_pane() {
    // ┌─────┬─────┐                   ┌─────┬─────┐
    // │     │     │                   │     │     │
    // ├─────┼─────┤                   ├─────┴─┬───┤
    // │█████│     │ ==resize=right==> │███████│   │
    // ├─────┼─────┤                   ├─────┬─┴───┤
    // │     │     │                   │     │     │
    // └─────┴─────┘                   └─────┴─────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 20,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_HORIZONTALLY,
        &SPLIT_HORIZONTALLY,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_left_aligned_top_and_bottom_with_panes_above_and_below() {
    // ┌─────┬─────┐                   ┌─────┬─────┐
    // ├─────┼─────┤                   ├─────┴─┬───┤
    // │     ├─────┤                   │       ├───┤
    // │     │█████│ ==resize=right==> │       │███│
    // │     ├─────┤                   │       ├───┤
    // ├─────┼─────┤                   ├─────┬─┴───┤
    // └─────┴─────┘                   └─────┴─────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 40,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_HORIZONTALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &RESIZE_UP,
        &RESIZE_UP,
        &RESIZE_UP,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &SPLIT_HORIZONTALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_UP,
        &RESIZE_UP,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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

#[test]
pub fn resize_right_with_panes_to_the_right_aligned_top_and_bottom_with_panes_above_and_below() {
    // ┌─────┬─────┐                   ┌─────┬─────┐
    // ├─────┼─────┤                   ├─────┴─┬───┤
    // ├─────┤     │                   ├───────┤   │
    // │█████│     │ ==resize=right==> │███████│   │
    // ├─────┤     │                   ├───────┤   │
    // ├─────┼─────┤                   ├─────┬─┴───┤
    // └─────┴─────┘                   └─────┴─────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 40,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);

    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &SPLIT_HORIZONTALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &RESIZE_UP,
        &RESIZE_UP,
        &RESIZE_UP,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_VERTICALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &SPLIT_HORIZONTALLY,
        &SPLIT_HORIZONTALLY,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &MOVE_FOCUS,
        &RESIZE_UP,
        &RESIZE_UP,
        &RESIZE_RIGHT,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

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
