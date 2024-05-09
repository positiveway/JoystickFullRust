use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use color_eyre::eyre::bail;
use universal_input::{InputEmulator, KeyCode};
use crate::buttons_state::{ButtonsState, Command};
use crate::configs::MainConfigs;
use crate::exec_or_eyre;
use crate::match_event::ButtonName;
use crate::math_ops::{ZoneAllowedRange, ZonesMapper};
use crate::pads_ops::{Coords, CoordsState, discard_jitter_for_pad, discard_jitter_for_stick, MouseMode, PadsCoords};
use crate::process_event::{ButtonEvent, ButtonReceiver, MouseEvent, MouseReceiver, PadStickEvent};

#[inline]
fn assign_pad_event(
    coords_state: &mut CoordsState,
    jitter_threshold: f32,
    pad_stick_event: PadStickEvent,
) {
    match pad_stick_event {
        PadStickEvent::FingerLifted => coords_state.reset(),
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
    coords_state: &mut CoordsState,
    jitter_threshold: f32,
    pad_stick_event: PadStickEvent,
) -> color_eyre::Result<()> {
    match pad_stick_event {
        PadStickEvent::FingerLifted => bail!("Cannot happen"),
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter_for_stick(coords_state.prev.x, value, jitter_threshold);
            // println!("X: {value}")
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter_for_stick(coords_state.prev.y, value, jitter_threshold);
            // println!("Y: {value}")
        }
    }

    if coords_state.any_changes() {
        let cur_pos = coords_state.cur_pos();
        let zero_coords = Coords { x: Some(0.0), y: Some(0.0) };
        if cur_pos == zero_coords {
            // if coords_state.cur.x == Some(0.0) || coords_state.cur.y == Some(0.0){
            //Finger lifted
            coords_state.reset();
        }
    }

    Ok(())
}

fn writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
) -> color_eyre::Result<()> {
    //Loading Configs
    let writing_interval = configs.mouse_refresh_interval;
    let layout_configs = configs.layout_configs;
    let gaming_mode = layout_configs.general.gaming_mode;
    let scroll_configs = layout_configs.scroll;
    let mouse_speed = layout_configs.general.mouse_speed;

    let mut pads_coords = PadsCoords::new(&layout_configs.finger_rotation_cfg);

    let mut buttons_state = ButtonsState::new(
        layout_configs.buttons_layout.clone(),
        layout_configs.general.repeat_keys,
    );

    //Zone Mapping
    let WASD_configs = layout_configs.wasd;
    let stick_zones_configs = layout_configs.stick;
    let _buttons_layout = layout_configs.buttons_layout.layout;

    let _wasd_zones: [Vec<KeyCode>; 4] = [
        vec![KeyCode::KEY_W],
        vec![KeyCode::KEY_A],
        vec![KeyCode::KEY_S],
        vec![KeyCode::KEY_D],
    ];
    let _wasd_zone_range = ZoneAllowedRange::from_one_value(WASD_configs.zone_range, WASD_configs.diagonal_zones)?;
    let mut wasd_zone_mapper = ZonesMapper::gen_from(
        _wasd_zones.to_vec(),
        90,
        &_wasd_zone_range,
        WASD_configs.start_threshold,
        WASD_configs.diagonal_zones,
    )?;

    let _stick_zones: [Vec<KeyCode>; 4] = [
        _buttons_layout[&ButtonName::BtnRight_SideL].clone(),
        _buttons_layout[&ButtonName::BtnUp_SideL].clone(),
        _buttons_layout[&ButtonName::BtnLeft_SideL].clone(),
        _buttons_layout[&ButtonName::BtnDown_SideL].clone(),
    ];
    let _stick_zone_range = ZoneAllowedRange::from_one_value(stick_zones_configs.zone_range, stick_zones_configs.diagonal_zones)?;
    let mut stick_zone_mapper = ZonesMapper::gen_from(
        _stick_zones.to_vec(),
        0,
        &_stick_zone_range,
        stick_zones_configs.start_threshold,
        stick_zones_configs.diagonal_zones,
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
                    pads_coords.reset();
                }
                MouseEvent::LeftPad(pad_stick_event) => assign_pad_event(
                    &mut pads_coords.left_pad,
                    layout_configs.jitter_threshold_cfg.left_pad,
                    pad_stick_event,
                ),
                MouseEvent::RightPad(pad_stick_event) => assign_pad_event(
                    &mut pads_coords.right_pad,
                    layout_configs.jitter_threshold_cfg.right_pad,
                    pad_stick_event,
                ),
                MouseEvent::Stick(pad_stick_event) => {
                    assign_stick_event(
                        &mut pads_coords.stick,
                        layout_configs.jitter_threshold_cfg.stick,
                        pad_stick_event,
                    )?;
                },
            }
        }

        // pads_coords.set_prev_if_cur_is_none();

        pads_coords.stick.send_commands_diff(
            &mut stick_zone_mapper,
            &stick_zones_configs,
            &mut buttons_state,
            false,
        )?;

        if mouse_mode != MouseMode::Typing {
            if pads_coords.right_pad.any_changes() {
                let mouse_diff = pads_coords.right_pad.diff();
                let mouse_diff = mouse_diff.convert(mouse_speed);
                if mouse_diff.is_any_changes() {
                    input_emulator.move_mouse(mouse_diff.x, -mouse_diff.y)?;
                }
            }
            match gaming_mode {
                false => {
                    if pads_coords.left_pad.any_changes() {
                        let mut scroll_diff = pads_coords.left_pad.diff();
                        if scroll_diff.x.abs() <= scroll_configs.horizontal_threshold {
                            scroll_diff.x = 0.0;
                        } else {
                            scroll_diff.y = 0.0;
                        }

                        let scroll_diff = scroll_diff.convert(scroll_configs.speed);
                        if scroll_diff.is_any_changes() {
                            exec_or_eyre!(input_emulator.scroll_x(scroll_diff.x))?;
                            exec_or_eyre!(input_emulator.scroll_y(-scroll_diff.y))?;
                        }
                    }
                }
                true => {
                    const ALWAYS_PRESS: bool = false; //For DEBUG purposes

                    pads_coords.left_pad.send_commands_diff(
                        &mut wasd_zone_mapper,
                        &WASD_configs,
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

pub fn create_writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
) -> JoinHandle<()> {
    thread::spawn(move || {
        writing_thread(mouse_receiver, button_receiver, configs).unwrap();
    })
}
