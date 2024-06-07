use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use color_eyre::eyre::{bail, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use universal_input::{InputEmulator, KeyCode, OS_Input_Coord, EventParams};
use crate::buttons_state::{ButtonsState, Command};
use crate::configs::MainConfigs;
use crate::exec_or_eyre;
use crate::match_event::ButtonName;
use crate::math_ops::{ZoneAllowedRange, ZonesMapper};
use crate::pads_ops::{ConvertedCoordsDiff, Coords, CoordsHistoryState, discard_jitter_for_pad, discard_jitter_for_stick, MouseMode, PadsCoords};
use crate::pads_ops::CoordState::Value;
use crate::process_event::{ButtonEvent, ButtonReceiver, MouseEvent, MouseReceiver, PadStickEvent};
use crate::utils::{check_thread_handle, ThreadHandle, ThreadHandleOption};

#[inline]
fn assign_pad_event(
    coords_state: &mut CoordsHistoryState,
    pad_stick_event: PadStickEvent,
) {
    let (zero_x, zero_y) = (coords_state.zero_x, coords_state.zero_y);

    let jitter_threshold = coords_state.jitter_threshold;

    match pad_stick_event {
        PadStickEvent::FingerLifted => {
            coords_state.set_to_discard_next();
            coords_state.new_x = zero_x;
            coords_state.new_y = zero_y;
            debug!("\nFinger lifted\n")
        },
        PadStickEvent::FingerPut => {
            coords_state.reset_all();
            coords_state.new_x = zero_x;
            coords_state.new_y = zero_y;
            debug!("\nFinger put\n")
        },
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter_for_pad(coords_state.prev.x, value, jitter_threshold);
            coords_state.new_x = value;
            // println!("X: {value}")
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter_for_pad(coords_state.prev.y, value, jitter_threshold);
            coords_state.new_y = value;
            // println!("Y: {value}")
        }
    }
}

#[inline]
fn assign_stick_event(
    coords_state: &mut CoordsHistoryState,
    pad_stick_event: PadStickEvent,
) -> Result<()> {
    let (zero_x, zero_y) = (coords_state.zero_x, coords_state.zero_y);

    let jitter_threshold = coords_state.jitter_threshold;

    match pad_stick_event {
        PadStickEvent::FingerLifted | PadStickEvent::FingerPut => bail!("Cannot happen"),
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter_for_stick(
                coords_state.prev.x,
                value,
                jitter_threshold,
                zero_x,
            );
            // println!("X: {value}")
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter_for_stick(
                coords_state.prev.y,
                value,
                jitter_threshold,
                zero_y,
            );
            // println!("Y: {value}")
        }
    }

    let zero_coords = Coords {
        x: Value(zero_x),
        y: Value(zero_y),
    };

    if coords_state.any_changes() {
        let cur_pos = coords_state.cur_pos();
        if cur_pos == zero_coords {
            // if coords_state.cur.x == Some(0.0) || coords_state.cur.y == Some(0.0){
            //Finger lifted
            coords_state.reset_all();
        }
    }

    Ok(())
}

pub fn write_events(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
    thread_handle: ThreadHandleOption,
) -> Result<()> {
    //Loading Configs
    let writing_interval = configs.general.mouse_refresh_interval;
    let layout_configs = configs.layout_configs;
    let gaming_mode = layout_configs.general.gaming_mode;
    let scroll_cfg = layout_configs.scroll_cfg;
    let mouse_speed = layout_configs.general.mouse_speed;

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

    let mut write_buffer: Vec<EventParams> = vec![];

    loop {
        let loop_start_time = Instant::now();

        if check_thread_handle(thread_handle).is_err() {
            return Ok(())
        };

        //MOUSE
        for event in mouse_receiver.try_iter() {
        //TODO: test try_recv_realtime. fallback: try_recv()
            // while let Some(event) = mouse_receiver.try_recv()? {
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

        #[cfg(feature = "use_only_last_coords")]{
            pads_coords.left_pad.cur.x = discard_jitter_for_pad(
                pads_coords.left_pad.prev.x,
                pads_coords.left_pad.new_x,
                pads_coords.left_pad.jitter_threshold,
            );
            pads_coords.left_pad.cur.y = discard_jitter_for_pad(
                pads_coords.left_pad.prev.y,
                pads_coords.left_pad.new_y,
                pads_coords.left_pad.jitter_threshold,
            );
            pads_coords.right_pad.cur.x = discard_jitter_for_pad(
                pads_coords.right_pad.prev.x,
                pads_coords.right_pad.new_x,
                pads_coords.right_pad.jitter_threshold,
            );
            pads_coords.right_pad.cur.y = discard_jitter_for_pad(
                pads_coords.right_pad.prev.y,
                pads_coords.right_pad.new_y,
                pads_coords.right_pad.jitter_threshold,
            );
        }

        // pads_coords.set_prev_if_cur_is_none();

        pads_coords.stick.send_commands_diff(
            &mut stick_zone_mapper,
            &stick_zones_cfg,
            &mut buttons_state,
        )?;

        if mouse_mode != MouseMode::Typing {
            if pads_coords.right_pad.any_changes() {
                let mouse_diff = pads_coords.right_pad.diff();
                let mouse_diff = mouse_diff.convert(mouse_speed);
                if mouse_diff.is_any_changes() {
                    #[cfg(feature = "gradual_mouse")]{
                        // println!("Gradual Mouse");
                        #[cfg(feature = "use_buffered_input")]{
                            write_buffer.extend(input_emulator.buffered_gradual_move_mouse(mouse_diff.x, mouse_diff.y));
                        }
                        #[cfg(not(feature = "use_buffered_input"))]{
                            #[cfg(feature = "use_raw_input")]{
                                input_emulator.gradual_move_mouse_raw(mouse_diff.x, mouse_diff.y)?;
                            }
                            #[cfg(not(feature = "use_raw_input"))]{
                                input_emulator.gradual_move_mouse(mouse_diff.x, mouse_diff.y)?;
                            }
                        }
                    }
                    #[cfg(not(feature = "gradual_mouse"))]{
                        input_emulator.move_mouse(mouse_diff.x, mouse_diff.y)?;
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
                            #[cfg(feature = "gradual_scroll")]{
                                // println!("Gradual Scroll");
                                #[cfg(feature = "use_buffered_input")]{
                                    write_buffer.extend(input_emulator.buffered_gradual_scroll(scroll_diff.x, scroll_diff.y));
                                }
                                #[cfg(not(feature = "use_buffered_input"))]{
                                    #[cfg(feature = "use_raw_input")]{
                                        input_emulator.gradual_scroll_raw(scroll_diff.x, scroll_diff.y)?;
                                    }
                                    #[cfg(not(feature = "use_raw_input"))]{
                                        input_emulator.gradual_scroll(scroll_diff.x, scroll_diff.y)?;
                                    }
                                }
                            }
                            #[cfg(not(feature = "gradual_scroll"))]{
                                if scroll_diff.x != 0 {
                                    input_emulator.scroll_x(scroll_diff.x)?;
                                }
                                if scroll_diff.y != 0 {
                                    input_emulator.scroll_y(scroll_diff.y)?;
                                }
                            }
                        }
                    }
                }
                true => {
                    pads_coords.left_pad.send_commands_diff(
                        &mut wasd_zone_mapper,
                        &WASD_zones_cfg,
                        &mut buttons_state,
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
        for event in button_receiver.try_iter() {
        //TODO: test try_recv_realtime. fallback: try_recv()
            // while let Some(event) = button_receiver.try_recv()? {
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

        #[cfg(feature = "use_buffered_input")]{
            for command in &buttons_state.queue {
                match command {
                    Command::Pressed(key_code) => {
                        // println!("Send Pressed: {}", key_code);
                        write_buffer.extend(input_emulator.buffered_press(*key_code)?);
                    }
                    Command::Released(key_code) => {
                        // println!("Send Released: {}", key_code);
                        write_buffer.extend(input_emulator.buffered_release(*key_code)?);
                    }
                }
            }
        }
        #[cfg(not(feature = "use_buffered_input"))]{
            for command in &buttons_state.queue {
                match command {
                    Command::Pressed(key_code) => {
                        input_emulator.press(*key_code)?;
                    }
                    Command::Released(key_code) => {
                        input_emulator.release(*key_code)?;
                    }
                }
            }
        }

        buttons_state.queue.clear();

        #[cfg(feature = "use_buffered_input")]{
            input_emulator.write_buffer(&write_buffer)?;
            write_buffer.clear()
        }

        //Scheduler
        let loop_iteration_runtime = loop_start_time.elapsed();

        if let Some(remaining) = writing_interval.checked_sub(loop_iteration_runtime) {
            sleep(remaining);
        }
    }
}



pub fn create_writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
) -> ThreadHandle {
    thread::spawn(move || {
        write_events(mouse_receiver, button_receiver, configs, None).unwrap()
    })
}
