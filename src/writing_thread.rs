use std::cmp::min;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use color_eyre::eyre::{bail, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use universal_input::{InputEmulator, KeyCode, OS_Input_Coord};
use crate::buttons_state::{ButtonsState, Command};
use crate::configs::MainConfigs;
use crate::exec_or_eyre;
use crate::match_event::ButtonName;
use crate::math_ops::{ZoneAllowedRange, ZonesMapper};
use crate::pads_ops::{ConvertedCoordsDiff, Coords, CoordsHistoryState, discard_jitter_for_pad, discard_jitter_for_stick, MouseMode, PadsCoords};
use crate::pads_ops::CoordState::Value;
use crate::process_event::{ButtonEvent, ButtonReceiver, MouseEvent, MouseReceiver, PadStickEvent};

#[inline]
fn assign_pad_event(
    coords_state: &mut CoordsHistoryState,
    pad_stick_event: PadStickEvent,
) {
    let jitter_threshold = coords_state.jitter_threshold;

    match pad_stick_event {
        PadStickEvent::FingerLifted => {
            coords_state.set_to_discard_next();
            debug!("\nFinger lifted\n")
        },
        PadStickEvent::FingerPut => {
            coords_state.reset_all();
            debug!("\nFinger put\n")
        },
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter_for_pad(coords_state.prev.x, value, jitter_threshold);
            // println!("X: {value}")
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter_for_pad(coords_state.prev.y, value, jitter_threshold);
            // println!("Y: {value}")
        }
    }
}

#[inline]
fn assign_stick_event(
    coords_state: &mut CoordsHistoryState,
    pad_stick_event: PadStickEvent,
) -> Result<()> {
    let axis_correction = coords_state.axis_correction;
    let use_correction = coords_state.use_correction;
    let jitter_threshold = coords_state.jitter_threshold;

    match pad_stick_event {
        PadStickEvent::FingerLifted | PadStickEvent::FingerPut => bail!("Cannot happen"),
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter_for_stick(
                coords_state.prev.x,
                value,
                jitter_threshold,
                axis_correction.x,
                use_correction,
            );
            // println!("X: {value}")
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter_for_stick(
                coords_state.prev.y,
                value,
                jitter_threshold,
                axis_correction.y,
                use_correction,
            );
            // println!("Y: {value}")
        }
    }

    if coords_state.any_changes() {
        let cur_pos = coords_state.cur_pos();
        let zero_coords = match use_correction {
            true => Coords {
                x: Value(axis_correction.x as f32),
                y: Value(axis_correction.y as f32),
            },
            false => Coords {
                x: Value(0.0),
                y: Value(0.0),
            }
        };

        if cur_pos == zero_coords {
            // if coords_state.cur.x == Some(0.0) || coords_state.cur.y == Some(0.0){
            //Finger lifted
            coords_state.reset_all();
        }
    }

    Ok(())
}

#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct GradualMove {
    pub x_direction: OS_Input_Coord,
    pub y_direction: OS_Input_Coord,
    pub both_move: OS_Input_Coord,
    pub move_only_x: OS_Input_Coord,
    pub move_only_y: OS_Input_Coord,
}

impl GradualMove {
    pub fn calculate(mouse_diff: ConvertedCoordsDiff) -> Self {
        // println!("Diff X: {}, Diff Y: {}", mouse_diff.x, mouse_diff.y);

        let x_direction = mouse_diff.x.signum();
        let y_direction = mouse_diff.y.signum();

        let move_x = mouse_diff.x.abs();
        let move_y = mouse_diff.y.abs();

        let both_move = min(move_x, move_y);

        // println!("Dir X: {}, Dir Y: {}, Move both: {}", x_direction, y_direction, both_move);

        let move_only_x = move_x - both_move;
        let move_only_y = move_y - both_move;

        // println!("Only X: {}, Only Y: {}\n", move_only_x, move_only_y);

        Self {
            x_direction,
            y_direction,
            both_move,
            move_only_x,
            move_only_y,
        }
    }
}

fn writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
) -> Result<()> {
    //Loading Configs
    let writing_interval = configs.mouse_refresh_interval;
    let layout_configs = configs.layout_configs;
    let gaming_mode = layout_configs.general.gaming_mode;
    let scroll_cfg = layout_configs.scroll_cfg;
    let mouse_speed = layout_configs.general.mouse_speed;
    let gradual_move_cfg = layout_configs.gradual_move_cfg;

    let mut pads_coords = PadsCoords::new(
        &layout_configs.finger_rotation_cfg,
        &layout_configs.axis_correction_cfg,
        &layout_configs.jitter_threshold_cfg,
    );

    let mut buttons_state = ButtonsState::new(
        layout_configs.buttons_layout.clone(),
        layout_configs.general.repeat_keys,
    );

    //Zone Mapping
    let WASD_zones_cfg = layout_configs.wasd_zones_cfg;
    let stick_zones_cfg = layout_configs.stick_zones_cfg;
    let _buttons_layout = layout_configs.buttons_layout.layout;

    let _wasd_zones: [Vec<KeyCode>; 4] = [
        vec![KeyCode::KEY_W],
        vec![KeyCode::KEY_A],
        vec![KeyCode::KEY_S],
        vec![KeyCode::KEY_D],
    ];
    let _wasd_zone_range = ZoneAllowedRange::from_one_value(WASD_zones_cfg.zone_range, WASD_zones_cfg.diagonal_zones)?;
    let mut wasd_zone_mapper = ZonesMapper::gen_from(
        _wasd_zones.to_vec(),
        90,
        &_wasd_zone_range,
        WASD_zones_cfg.start_threshold,
        WASD_zones_cfg.diagonal_zones,
    )?;

    let _stick_zones: [Vec<KeyCode>; 4] = [
        _buttons_layout[&ButtonName::BtnRight_SideL].clone(),
        _buttons_layout[&ButtonName::BtnUp_SideL].clone(),
        _buttons_layout[&ButtonName::BtnLeft_SideL].clone(),
        _buttons_layout[&ButtonName::BtnDown_SideL].clone(),
    ];
    let _stick_zone_range = ZoneAllowedRange::from_one_value(stick_zones_cfg.zone_range, stick_zones_cfg.diagonal_zones)?;
    let mut stick_zone_mapper = ZonesMapper::gen_from(
        _stick_zones.to_vec(),
        0,
        &_stick_zone_range,
        stick_zones_cfg.start_threshold,
        stick_zones_cfg.diagonal_zones,
    )?;
    //Zone Mapping
    //Loading Configs

    let mut input_emulator = InputEmulator::new()?;
    let mut mouse_mode = MouseMode::default();

    loop {
        let start = Instant::now();

        //MOUSE
        // for event in mouse_receiver.try_iter() {
        //TODO: test try_recv_realtime. fallback: try_recv()
        while let Some(event) = mouse_receiver.try_recv()? {
            match event {
                MouseEvent::ModeSwitched => match mouse_mode {
                    MouseMode::CursorMove => {
                        mouse_mode = MouseMode::Typing;
                    }
                    MouseMode::Typing => {
                        mouse_mode = MouseMode::CursorMove;
                    }
                },
                MouseEvent::Reset => {
                    mouse_mode = MouseMode::default();
                    pads_coords.reset_all();
                }
                MouseEvent::LeftPad(pad_stick_event) => assign_pad_event(
                    &mut pads_coords.left_pad,
                    pad_stick_event,
                ),
                MouseEvent::RightPad(pad_stick_event) => assign_pad_event(
                    &mut pads_coords.right_pad,
                    pad_stick_event,
                ),
                MouseEvent::Stick(pad_stick_event) => {
                    assign_stick_event(
                        &mut pads_coords.stick,
                        pad_stick_event,
                    )?;
                },
            }
        }

        // pads_coords.set_prev_if_cur_is_none();

        pads_coords.stick.send_commands_diff(
            &mut stick_zone_mapper,
            &stick_zones_cfg,
            &mut buttons_state,
            false,
        )?;

        if mouse_mode != MouseMode::Typing {
            if pads_coords.right_pad.any_changes() {
                let mouse_diff = pads_coords.right_pad.diff();
                let mouse_diff = mouse_diff.convert(mouse_speed);
                if mouse_diff.is_any_changes() {
                    match gradual_move_cfg.mouse {
                        true => {
                            // println!("Gradual Mouse");
                            let gradual_move = GradualMove::calculate(mouse_diff);

                            for _ in 0..gradual_move.both_move {
                                input_emulator.move_mouse(gradual_move.x_direction, gradual_move.y_direction)?;
                            }
                            for _ in 0..gradual_move.move_only_x {
                                input_emulator.move_mouse_x(gradual_move.x_direction)?;
                            }
                            for _ in 0..gradual_move.move_only_y {
                                input_emulator.move_mouse_y(gradual_move.y_direction)?;
                            }
                        }
                        false => {
                            input_emulator.move_mouse(mouse_diff.x, mouse_diff.y)?;
                        }
                    }
                }
            }
            match gaming_mode {
                false => {
                    if pads_coords.left_pad.any_changes() {
                        let mut scroll_diff = pads_coords.left_pad.diff();
                        if scroll_diff.x.abs() <= scroll_cfg.horizontal_threshold {
                            scroll_diff.x = 0.0;
                        } else {
                            scroll_diff.y = 0.0;
                        }

                        let scroll_diff = scroll_diff.convert(scroll_cfg.speed);
                        if scroll_diff.is_any_changes() {
                            match gradual_move_cfg.scroll {
                                true => {
                                    // println!("Gradual Scroll");
                                    let gradual_scroll = GradualMove::calculate(scroll_diff);

                                    for _ in 0..gradual_scroll.both_move {
                                        input_emulator.scroll_x(gradual_scroll.x_direction)?;
                                        input_emulator.scroll_y(gradual_scroll.y_direction)?;
                                    }
                                    for _ in 0..gradual_scroll.move_only_x {
                                        input_emulator.scroll_x(gradual_scroll.x_direction)?;
                                    }
                                    for _ in 0..gradual_scroll.move_only_y {
                                        input_emulator.scroll_y(gradual_scroll.y_direction)?;
                                    }
                                }
                                false => {
                                    input_emulator.scroll_x(scroll_diff.x)?;
                                    input_emulator.scroll_y(scroll_diff.y)?;
                                }
                            }
                        }
                    }
                }
                true => {
                    const ALWAYS_PRESS: bool = false; //For DEBUG purposes

                    pads_coords.left_pad.send_commands_diff(
                        &mut wasd_zone_mapper,
                        &WASD_zones_cfg,
                        &mut buttons_state,
                        ALWAYS_PRESS,
                    )?;
                }
            }
        }

        // pads_coords.stick.update();
        //Important to keep
        // pads_coords.update_if_not_init();
        pads_coords.update();
        pads_coords.reset_current();

        //BUTTONS
        // for event in button_receiver.try_iter() {
        //TODO: test try_recv_realtime. fallback: try_recv()
        while let Some(event) = button_receiver.try_recv()? {
            match event {
                //Press goes first to check if already pressed
                ButtonEvent::Pressed(button_name) => {
                    buttons_state.press(button_name, false)?;
                }
                ButtonEvent::Released(button_name) => {
                    buttons_state.release(button_name)?;
                }
            }
        }

        for command in &buttons_state.queue {
            match command {
                Command::Pressed(key_code) => {
                    // println!("Send Pressed: {}", key_code);
                    exec_or_eyre!(input_emulator.press(*key_code))?
                }
                Command::Released(key_code) => {
                    // println!("Send Released: {}", key_code);
                    exec_or_eyre!(input_emulator.release(*key_code))?
                }
            }
        }
        buttons_state.queue.clear();

        //Scheduler
        let runtime = start.elapsed();

        if let Some(remaining) = writing_interval.checked_sub(runtime) {
            sleep(remaining);
        }
    }
}

pub type ThreadHandle = JoinHandle<()>;

pub fn check_thread_handle(thread_handle: &ThreadHandle) -> Result<()> {
    if thread_handle.is_finished() {
        bail!("Thread panicked")
    } else {
        Ok(())
    }
}

pub fn try_unwrap_thread(thread_handle: ThreadHandle) {
    if thread_handle.is_finished() {
        thread_handle.join().unwrap();
    };
}

pub fn create_writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
) -> ThreadHandle {
    thread::spawn(move || {
        writing_thread(mouse_receiver, button_receiver, configs).unwrap()
    })
}
