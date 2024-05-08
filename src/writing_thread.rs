use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use universal_input::{InputEmulator, KeyCode};
use universal_input::KeyCode::KEY_LEFTSHIFT;
use crate::buttons_state::{ButtonsState, Command};
use crate::configs::MainConfigs;
use crate::exec_or_eyre;
use crate::math_ops::{ZoneAllowedRange, ZonesMapper};
use crate::pads_ops::{CoordsState, discard_jitter, MouseMode, PadsCoords};
use crate::process_event::{ButtonEvent, ButtonReceiver, MouseEvent, MouseReceiver, PadStickEvent};

#[inline]
fn assign_pad_stick_event(
    coords_state: &mut CoordsState,
    jitter_threshold: f32,
    pad_stick_event: PadStickEvent,
) {
    match pad_stick_event {
        PadStickEvent::FingerLifted => coords_state.reset(),
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter(coords_state.prev.x, value, jitter_threshold);
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter(coords_state.prev.y, value, jitter_threshold);
        }
    }
}

fn writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: MainConfigs,
) -> color_eyre::Result<()> {
    let mut input_emulator = InputEmulator::new()?;

    let writing_interval = configs.mouse_refresh_interval;
    let layout_configs = configs.layout_configs;
    let gaming_mode = layout_configs.general.gaming_mode;
    let scroll_configs = layout_configs.scroll;
    let mouse_speed = layout_configs.general.mouse_speed;
    let use_shift_movement = layout_configs.movement.use_shift;
    let zone_range = layout_configs.zone_range;

    let mut buttons_state = ButtonsState::new(
        layout_configs.buttons_layout,
        layout_configs.general.repeat_keys,
    );

    let mut mouse_mode = MouseMode::default();
    let mut pads_coords = PadsCoords::new(&layout_configs.finger_rotation);

    let _wasd_zones: [Vec<KeyCode>; 4] = [
        vec![KeyCode::KEY_W],
        vec![KeyCode::KEY_A],
        vec![KeyCode::KEY_S],
        vec![KeyCode::KEY_D],
    ];
    let _wasd_zone_range = ZoneAllowedRange::from_one_value(zone_range.wasd)?;
    let mut wasd_zone_mapper = ZonesMapper::gen_from_4_into_8(
        _wasd_zones,
        90,
        &_wasd_zone_range,
        layout_configs.movement.start_threshold,
    )?;

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
                MouseEvent::LeftPad(pad_stick_event) => assign_pad_stick_event(
                    &mut pads_coords.left_pad,
                    layout_configs.jitter_threshold.left_pad,
                    pad_stick_event,
                ),
                MouseEvent::RightPad(pad_stick_event) => assign_pad_stick_event(
                    &mut pads_coords.right_pad,
                    layout_configs.jitter_threshold.right_pad,
                    pad_stick_event,
                ),
                MouseEvent::Stick(pad_stick_event) => assign_pad_stick_event(
                    &mut pads_coords.stick,
                    layout_configs.jitter_threshold.stick,
                    pad_stick_event,
                ),
            }
        }

        // pads_coords.set_prev_if_cur_is_none();

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

                    let cur_pos = pads_coords
                        .left_pad
                        .cur_pos()
                        .try_rotate(pads_coords.left_pad.finger_rotation);

                    let (to_release, to_press, to_press_full) =
                        wasd_zone_mapper.get_commands_diff(cur_pos.x, cur_pos.y);
                    // if !to_release.is_empty() || !to_press.is_empty() {
                    //     println!("To release: '{:?}'; To press: '{:?}'", to_release, to_press)
                    // }

                    let to_press = if ALWAYS_PRESS {
                        to_press_full
                    } else {
                        to_press
                    };

                    //Press goes first to check if already pressed
                    for keycode in to_press {
                        buttons_state.press_keycodes(vec![keycode], ALWAYS_PRESS)?;
                    }
                    for keycode in to_release {
                        buttons_state.release_keycodes(vec![keycode], false)?;
                    }

                    if use_shift_movement {
                        if cur_pos.magnitude() > layout_configs.movement.shift_threshold {
                            buttons_state.press_keycodes(vec![KEY_LEFTSHIFT], ALWAYS_PRESS)?;
                        } else {
                            buttons_state.release_keycodes(vec![KEY_LEFTSHIFT], false)?;
                        }
                    }
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
