use crate::buttons_state::{ButtonsState, Command};
use crate::configs::{FingerRotation, JitterThreshold, MainConfigs};
use crate::exec_or_eyre;
use crate::key_codes::KeyCode::KEY_LEFTSHIFT;
use crate::key_codes::{key_codes_to_buttons, KeyCode};
use crate::math_ops::{rotate_around_center, Vector, ZoneAllowedRange, ZonesMapper};
use crate::process_event::{ButtonEvent, ButtonReceiver, MouseEvent, MouseReceiver, PadStickEvent};
use color_eyre::eyre::{bail, eyre, Result};
use log::{debug, info};
use mouse_keyboard_input::{Button, Coord, VirtualDevice};
use serde::{Deserialize, Serialize};
use std::thread;
use std::thread::{sleep, JoinHandle};
use std::time::Instant;
use strum_macros::Display;

#[derive(Display, Eq, Hash, PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MouseMode {
    #[default]
    CursorMove,
    Typing,
}

#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Coords {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

impl Coords {
    pub fn reset(&mut self) {
        self.x = None;
        self.y = None;
    }

    fn update_one_coord(prev: &mut Option<f32>, new: Option<f32>) {
        if new.is_some() {
            *prev = new;
        }
    }

    fn update_one_if_not_init(prev: &mut Option<f32>, new: Option<f32>) {
        if prev.is_none() {
            Self::update_one_coord(prev, new);
        }
    }

    pub fn update(&mut self, new: &Self) {
        Self::update_one_coord(&mut self.x, new.x);
        Self::update_one_coord(&mut self.y, new.y);
    }

    pub fn update_if_not_init(&mut self, new: &Self) {
        Self::update_one_if_not_init(&mut self.x, new.x);
        Self::update_one_if_not_init(&mut self.y, new.y);
    }

    // fn set_one_coord(cur: &mut Option<f32>, prev: Option<f32>) {
    //     if cur.is_none() {
    //         *cur = prev;
    //     }
    // }

    // pub fn set_prev_if_cur_is_none(&mut self, prev: &Self) {
    //     Self::set_one_coord(&mut self.x, prev.x);
    //     Self::set_one_coord(&mut self.y, prev.y);
    // }

    pub fn any_is_not_init(&self) -> bool {
        self.x.is_none() || self.y.is_none()
    }

    pub fn any_changes(&self) -> bool {
        self.x.is_some() || self.y.is_some()
    }

    pub fn rotate(&self, rotation: i16) -> Option<Self> {
        Vector::from_coords(*self).and_then(|vector: Vector| {
            Some(rotate_around_center(vector, rotation as f32).as_coords())
        })
    }

    pub fn try_rotate(&self, rotation: i16) -> Self {
        self.rotate(rotation).unwrap_or(*self)
    }

    pub fn magnitude(&self) -> f32 {
        match (self.x, self.y) {
            (Some(x), Some(y)) => x.hypot(y),
            (_, _) => 0.0,
        }
    }
}

fn option_to_string(value: Option<f32>) -> String {
    match value {
        None => "None".to_string(),
        Some(value) => value.to_string(),
    }
}

impl std::fmt::Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[x: {}, y: {}]",
            option_to_string(self.x),
            option_to_string(self.y)
        )
    }
}

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
struct CoordsDiff {
    pub x: f32,
    pub y: f32,
}

impl CoordsDiff {
    pub fn convert(&self, multiplier: u16) -> ConvertedCoordsDiff {
        ConvertedCoordsDiff {
            x: convert_diff(self.x, multiplier),
            y: convert_diff(self.y, multiplier),
        }
    }

    pub fn is_any_changes(&self) -> bool {
        self.x != 0.0 || self.y != 0.0
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
struct ConvertedCoordsDiff {
    pub x: Coord,
    pub y: Coord,
}

impl ConvertedCoordsDiff {
    pub fn is_any_changes(&self) -> bool {
        self.x != 0 || self.y != 0
    }
}

fn calc_diff_one_coord(prev_coord: Option<f32>, cur_coord: Option<f32>) -> f32 {
    match (prev_coord, cur_coord) {
        (Some(prev_value), Some(cur_value)) => cur_value - prev_value,
        _ => 0.0,
    }
}

fn convert_diff(value: f32, multiplier: u16) -> Coord {
    (value * multiplier as f32).round() as Coord
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CoordsState {
    prev: Coords,
    cur: Coords,
    finger_rotation: i16,
    use_rotation: bool,
}

impl CoordsState {
    pub fn new(finger_rotation: i16, use_rotation: bool) -> Self {
        Self {
            prev: Default::default(),
            cur: Default::default(),
            finger_rotation,
            use_rotation,
        }
    }

    pub fn any_changes(&self) -> bool {
        self.cur.any_changes()
    }

    // pub fn set_prev_if_cur_is_none(&mut self) {
    //     self.cur.set_prev_if_cur_is_none(&self.prev);
    // }

    pub fn reset(&mut self) {
        self.prev.reset();
        self.cur.reset();
    }

    pub fn reset_current(&mut self) {
        self.cur.reset();
    }

    pub fn update(&mut self) {
        self.prev.update(&self.cur)
    }

    pub fn update_if_not_init(&mut self) {
        self.prev.update_if_not_init(&self.cur);
    }

    //Left for DEBUG reference
    // pub fn rotate_cur_coords(&self) -> Option<Coords> {
        // let cur_pos = self.cur_pos();
        // match (
        //     cur_pos.x,
        //     cur_pos.y,
        // ) {
        //     (Some(x), Some(y)) => {
        //         let point = Vector { x, y };
        //
        //         let orig_angle = point.angle();
        //         let rotated_vector = rotate_around_center(point, self.finger_rotation as f32);
        //
        //         let rotated_coords = rotated_vector.as_coords();
        //         debug!("Origin: {}", self.cur);
        //         debug!("Filled: {}", self.cur_pos());
        //         debug!("Rotated: {}", rotated_coords);
        //         debug!(
        //             "Angle: [Orig: {}, Shifted: {}; Rotation: {}]",
        //             orig_angle,
        //             rotated_vector.angle(),
        //             self.finger_rotation
        //         );
        //         debug!("");
        //         Some(rotated_coords)
        //     }
        //     _ => None,
        // }
    // }

    pub fn cur_pos(&self) -> Coords {
        Coords {
            x: self.cur.x.or(self.prev.x),
            y: self.cur.y.or(self.prev.y),
        }
    }

    pub fn diff(&mut self) -> CoordsDiff {
        let cur_coords = match self.use_rotation {
            true => self.cur_pos().try_rotate(self.finger_rotation),
            false => self.cur_pos(),
        };
        let prev_coords = match self.use_rotation {
            true => self.prev.try_rotate(self.finger_rotation),
            false => self.prev,
        };

        let diff = CoordsDiff {
            x: calc_diff_one_coord(prev_coords.x, cur_coords.x),
            y: calc_diff_one_coord(prev_coords.y, cur_coords.y),
        };
        diff
    }

    pub fn convert_diff(&mut self, multiplier: u16) -> ConvertedCoordsDiff {
        self.diff().convert(multiplier)
    }

    // pub fn diff_and_update(&mut self) -> CoordsDiff {
    //     let diff = self.diff();
    //     if diff.is_any_changes() {
    //         self.update();
    //     }
    //     diff
    // }
    //
    // pub fn convert_diff_and_update(&mut self, multiplier: u16) -> ConvertedCoordsDiff {
    //     let converted_diff = self.convert_diff(multiplier);
    //     if converted_diff.is_any_changes() {
    //         self.update();
    //     }
    //     converted_diff
    // }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PadsCoords {
    left_pad: CoordsState,
    right_pad: CoordsState,
    stick: CoordsState,
}

impl PadsCoords {
    pub fn new(finger_rotation: &FingerRotation) -> Self {
        Self {
            left_pad: CoordsState::new(finger_rotation.left_pad, finger_rotation.use_rotation),
            right_pad: CoordsState::new(finger_rotation.right_pad, finger_rotation.use_rotation),
            stick: CoordsState::new(finger_rotation.stick, finger_rotation.use_rotation),
        }
    }

    pub fn reset(&mut self) {
        self.left_pad.reset();
        self.right_pad.reset();
        self.stick.reset();
    }

    pub fn reset_current(&mut self) {
        self.left_pad.reset_current();
        self.right_pad.reset_current();
        self.stick.reset_current();
    }

    // pub fn set_prev_if_cur_is_none(&mut self) {
    //     self.left_pad.set_prev_if_cur_is_none();
    //     self.right_pad.set_prev_if_cur_is_none();
    //     self.stick.set_prev_if_cur_is_none();
    // }

    pub fn update(&mut self) {
        self.left_pad.update();
        self.right_pad.update();
        self.stick.update();
    }

    pub fn update_if_not_init(&mut self) {
        self.left_pad.update_if_not_init();
        self.right_pad.update_if_not_init();
        self.stick.update_if_not_init();
    }
}

fn is_jitter(value: f32, jitter_threshold: f32) -> bool {
    value.abs() <= jitter_threshold
}

fn discard_jitter(prev_value: Option<f32>, new_value: f32, jitter_threshold: f32) -> Option<f32> {
    match prev_value {
        None => Some(new_value),
        Some(prev_value) => {
            let diff = new_value - prev_value;
            match is_jitter(diff, jitter_threshold) {
                true => None,
                false => Some(new_value),
            }
        }
    }
}

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
) -> Result<()> {
    let mut virtual_device = exec_or_eyre!(VirtualDevice::default())?;

    let writing_interval = configs.mouse_refresh_interval;
    let layout_configs = configs.layout_configs;
    let gaming_mode = layout_configs.general.gaming_mode;
    let scroll_configs = layout_configs.scroll;
    let mouse_speed = layout_configs.general.mouse_speed;
    let use_shift_movement = layout_configs.movement.use_shift;

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
    let _wasd_zone_range = ZoneAllowedRange::new(22, 22, 22)?;
    let mut wasd_zone_mapper = ZonesMapper::gen_from_4_into_8(
        _wasd_zones,
        90,
        &_wasd_zone_range,
        layout_configs.movement.start_threshold,
    )?;

    loop {
        let start = Instant::now();

        //MOUSE
        for event in mouse_receiver.try_iter() {
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
                    exec_or_eyre!(virtual_device.move_mouse(mouse_diff.x, -mouse_diff.y))?;
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
                            exec_or_eyre!(virtual_device.scroll_x(scroll_diff.x))?;
                            exec_or_eyre!(virtual_device.scroll_y(-scroll_diff.y))?;
                        }
                    }
                }
                true => {
                    const ALWAYS_PRESS: bool = false; //For DEBUG purposes

                    let cur_pos = pads_coords.left_pad.cur_pos().try_rotate(pads_coords.left_pad.finger_rotation);

                    let (to_release, to_press, to_press_full) =
                        wasd_zone_mapper.get_commands_diff(cur_pos.x, cur_pos.y);
                    // if !to_release.is_empty() || !to_press.is_empty() {
                    //     println!("To release: '{:?}'; To press: '{:?}'", to_release, to_press)
                    // }

                    let to_press = if ALWAYS_PRESS { to_press_full } else { to_press };

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
        for event in button_receiver.try_iter() {
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
                    let button = key_code.as_button()?;
                    // println!("Send Pressed: {}", button);
                    exec_or_eyre!(virtual_device.press(button))?
                }
                Command::Released(key_code) => {
                    let button = key_code.as_button()?;
                    // println!("Send Released: {}", button);
                    exec_or_eyre!(virtual_device.release(button))?
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
