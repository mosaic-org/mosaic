use insta::assert_snapshot;

use crate::panes::PositionAndSize;
use crate::tests::fakes::FakeInputOutput;
use crate::tests::utils::{get_next_to_last_snapshot, get_output_frame_snapshots};
use crate::{start, CliArgs};

use crate::tests::utils::commands::{
    COMMAND_TOGGLE, ESC, MOVE_FOCUS_IN_PANE_MODE, PANE_MODE, QUIT, RESIZE_DOWN_IN_RESIZE_MODE,
    RESIZE_LEFT_IN_RESIZE_MODE, RESIZE_MODE, SPLIT_DOWN_IN_PANE_MODE, SPLIT_RIGHT_IN_PANE_MODE,
};

fn get_fake_os_input(fake_win_size: &PositionAndSize) -> FakeInputOutput {
    FakeInputOutput::new(*fake_win_size)
}

#[test]
pub fn resize_down_with_pane_above() {
    // ┌───────────┐                  ┌───────────┐
    // │           │                  │           │
    // │           │                  │           │
    // ├───────────┤ ==resize=down==> │           │
    // │███████████│                  ├───────────┤
    // │███████████│                  │███████████│
    // │███████████│                  │███████████│
    // └───────────┘                  └───────────┘
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
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_pane_below() {
    // ┌───────────┐                  ┌───────────┐
    // │███████████│                  │███████████│
    // │███████████│                  │███████████│
    // ├───────────┤ ==resize=down==> │███████████│
    // │           │                  ├───────────┤
    // │           │                  │           │
    // └───────────┘                  └───────────┘
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
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_above_and_below() {
    // ┌───────────┐                  ┌───────────┐
    // │           │                  │           │
    // │           │                  │           │
    // ├───────────┤                  │           │
    // │███████████│ ==resize=down==> ├───────────┤
    // │███████████│                  │███████████│
    // ├───────────┤                  ├───────────┤
    // │           │                  │           │
    // │           │                  │           │
    // └───────────┘                  └───────────┘
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
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_multiple_panes_above() {
    //
    // ┌─────┬─────┐                    ┌─────┬─────┐
    // │     │     │                    │     │     │
    // ├─────┴─────┤  ==resize=down==>  │     │     │
    // │███████████│                    ├─────┴─────┤
    // │███████████│                    │███████████│
    // └───────────┘                    └───────────┘
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
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_above_aligned_left_with_current_pane() {
    // ┌─────┬─────┐                    ┌─────┬─────┐
    // │     │     │                    │     │     │
    // │     │     │                    │     │     │
    // ├─────┼─────┤  ==resize=down==>  ├─────┤     │
    // │     │█████│                    │     ├─────┤
    // │     │█████│                    │     │█████│
    // └─────┴─────┘                    └─────┴─────┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_below_aligned_left_with_current_pane() {
    // ┌─────┬─────┐                    ┌─────┬─────┐
    // │     │█████│                    │     │█████│
    // │     │█████│                    │     │█████│
    // ├─────┼─────┤  ==resize=down==>  ├─────┤█████│
    // │     │     │                    │     ├─────┤
    // │     │     │                    │     │     │
    // └─────┴─────┘                    └─────┴─────┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_above_aligned_right_with_current_pane() {
    // ┌─────┬─────┐                    ┌─────┬─────┐
    // │     │     │                    │     │     │
    // │     │     │                    │     │     │
    // ├─────┼─────┤  ==resize=down==>  │     ├─────┤
    // │█████│     │                    ├─────┤     │
    // │█████│     │                    │█████│     │
    // └─────┴─────┘                    └─────┴─────┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_below_aligned_right_with_current_pane() {
    // ┌─────┬─────┐                    ┌─────┬─────┐
    // │█████│     │                    │█████│     │
    // │█████│     │                    │█████│     │
    // ├─────┼─────┤  ==resize=down==>  │█████├─────┤
    // │     │     │                    ├─────┤     │
    // │     │     │                    │     │     │
    // └─────┴─────┘                    └─────┴─────┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_above_aligned_left_and_right_with_current_pane() {
    // ┌───┬───┬───┐                    ┌───┬───┬───┐
    // │   │   │   │                    │   │   │   │
    // │   │   │   │                    │   │   │   │
    // ├───┼───┼───┤  ==resize=down==>  ├───┤   ├───┤
    // │   │███│   │                    │   ├───┤   │
    // │   │███│   │                    │   │███│   │
    // └───┴───┴───┘                    └───┴───┴───┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_below_aligned_left_and_right_with_current_pane() {
    // ┌───┬───┬───┐                    ┌───┬───┬───┐
    // │   │███│   │                    │   │███│   │
    // │   │███│   │                    │   │███│   │
    // ├───┼───┼───┤  ==resize=down==>  ├───┤███├───┤
    // │   │   │   │                    │   ├───┤   │
    // │   │   │   │                    │   │   │   │
    // └───┴───┴───┘                    └───┴───┴───┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_above_aligned_left_and_right_with_panes_to_the_left_and_right() {
    // ┌─┬───────┬─┐                    ┌─┬───────┬─┐
    // │ │       │ │                    │ │       │ │
    // │ │       │ │                    │ │       │ │
    // ├─┼─┬───┬─┼─┤  ==resize=down==>  ├─┤       ├─┤
    // │ │ │███│ │ │                    │ ├─┬───┬─┤ │
    // │ │ │███│ │ │                    │ │ │███│ │ │
    // └─┴─┴───┴─┴─┘                    └─┴─┴───┴─┴─┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn resize_down_with_panes_below_aligned_left_and_right_with_to_the_left_and_right() {
    // ┌─┬─┬───┬─┬─┐                    ┌─┬─┬───┬─┬─┐
    // │ │ │███│ │ │                    │ │ │███│ │ │
    // │ │ │███│ │ │                    │ │ │███│ │ │
    // ├─┼─┴───┴─┼─┤  ==resize=down==>  ├─┤ │███│ ├─┤
    // │ │       │ │                    │ ├─┴───┴─┤ │
    // │ │       │ │                    │ │       │ │
    // └─┴───────┴─┘                    └─┴───────┴─┘
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
        &PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &SPLIT_RIGHT_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &MOVE_FOCUS_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_LEFT_IN_RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);

    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}

#[test]
pub fn cannot_resize_down_when_pane_below_is_at_minimum_height() {
    // ┌───────────┐                  ┌───────────┐
    // │███████████│                  │███████████│
    // ├───────────┤ ==resize=down==> ├───────────┤
    // │           │                  │           │
    // └───────────┘                  └───────────┘
    // █ == focused pane
    let fake_win_size = PositionAndSize {
        columns: 121,
        rows: 5,
        x: 0,
        y: 0,
    };
    let mut fake_input_output = get_fake_os_input(&fake_win_size);
    fake_input_output.add_terminal_input(&[
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &PANE_MODE,
        &SPLIT_DOWN_IN_PANE_MODE,
        &ESC,
        &COMMAND_TOGGLE,
        &COMMAND_TOGGLE,
        &RESIZE_MODE,
        &RESIZE_DOWN_IN_RESIZE_MODE,
        &QUIT,
    ]);
    start(Box::new(fake_input_output.clone()), CliArgs::default());

    let output_frames = fake_input_output
        .stdout_writer
        .output_frames
        .lock()
        .unwrap();
    let snapshots = get_output_frame_snapshots(&output_frames, &fake_win_size);
    let snapshot_before_quit =
        get_next_to_last_snapshot(snapshots).expect("could not find snapshot");
    assert_snapshot!(snapshot_before_quit);
}
