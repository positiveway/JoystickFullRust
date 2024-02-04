use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use mouse_keyboard_input::{Coord, VirtualDevice};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::configs::{Configs, FingerRotation, JitterThreshold};
use color_eyre::eyre::{bail, Result};
use log::{debug, info};
use crate::process_event::{MouseEvent, MouseReceiver, ButtonReceiver, PadStickEvent, ButtonEvent};
use crate::exec_or_eyre;
use crate::math_ops::{hypot, rotate_around_center, Vector, NONE_VAL_ERR_MSG};


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

    pub fn any_is_none(&self) -> bool {
        self.x.is_none() || self.y.is_none()
    }

    pub fn any_changes(&self) -> bool {
        self.x.is_some() || self.y.is_some()
    }
}

fn option_to_string(value: Option<f32>) -> String {
    match value {
        None => {
            "None".to_string()
        }
        Some(value) => {
            value.to_string()
        }
    }
}

impl std::fmt::Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "[x: {}, y: {}]", option_to_string(self.x), option_to_string(self.y))
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

    pub fn magnitude(&self) -> f64 {
        hypot(self.x, self.y)
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

    pub fn magnitude(&self) -> f64 {
        hypot(self.x, self.y)
    }
}

fn calc_diff_one_coord(prev_coord: Option<f32>, cur_coord: Option<f32>) -> f32 {
    match prev_coord {
        None => { 0.0 }
        Some(prev_value) => {
            match cur_coord {
                None => { 0.0 }
                Some(cur_value) => {
                    cur_value - prev_value
                }
            }
        }
    }
}

fn convert_diff(value: f32, multiplier: u16) -> Coord {
    (value * multiplier as f32).round() as Coord
}


#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CoordsState {
    prev: Coords,
    cur: Coords,
    jitter_threshold: f32,
    finger_rotation: i16,
    use_rotation: bool,
}

impl CoordsState {
    pub fn new(jitter_threshold: f32, finger_rotation: i16, use_rotation: bool) -> Self {
        Self {
            prev: Default::default(),
            cur: Default::default(),
            jitter_threshold,
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

    fn get_cur_or_prev(prev: Option<f32>, cur: Option<f32>) -> Result<f32> {
        Ok(match cur {
            None => match prev {
                None => bail!(NONE_VAL_ERR_MSG),
                Some(value) => { value }
            },
            Some(value) => { value }
        })
    }

    pub fn rotate_cur_coords(&self) -> Result<Coords> {
        let point = Vector {
            x: Self::get_cur_or_prev(self.prev.x, self.cur.x)?,
            y: Self::get_cur_or_prev(self.prev.y, self.cur.y)?,
        };

        let orig_angle = point.angle();
        let rotated_vector = rotate_around_center(point, self.finger_rotation as f32);

        let rotated_coords = rotated_vector.as_coords();
        debug!("Origin: {}", self.cur);
        debug!("Rotated: {}", rotated_coords);
        debug!("Angle: [Orig: {}, Shifted: {}; Rotation: {}]",
                 orig_angle, rotated_vector.angle(), self.finger_rotation);
        debug!("");
        Ok(rotated_coords)
    }

    pub fn rotate_prev_coords(&self) -> Result<Coords> {
        Ok(rotate_around_center(Vector::from_coords(self.prev)?, self.finger_rotation as f32).as_coords())
    }

    pub fn diff(&mut self) -> CoordsDiff {
        let cur_coords = match self.use_rotation {
            true => {
                match self.rotate_cur_coords() {
                    Err(error) => {
                        info!("{}", error);
                        self.cur
                    }
                    Ok(rotated_coords) => {
                        rotated_coords
                    }
                }
            }
            false => {
                self.cur
            }
        };
        let prev_coords = match self.use_rotation {
            true => {
                match self.rotate_prev_coords() {
                    Ok(value) => { value }
                    Err(error) => {
                        info!("{}", error);
                        self.prev
                    }
                }
            }
            false => {
                self.prev
            }
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
    pub fn new(jitter_threshold: &JitterThreshold, finger_rotation: &FingerRotation, debug: bool) -> Self {
        Self {
            left_pad: CoordsState::new(
                jitter_threshold.left_pad, finger_rotation.left_pad, finger_rotation.use_rotation),
            right_pad: CoordsState::new(
                jitter_threshold.right_pad, finger_rotation.right_pad, finger_rotation.use_rotation),
            stick: CoordsState::new(
                jitter_threshold.stick, finger_rotation.stick, finger_rotation.use_rotation),
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
        None => { Some(new_value) }
        Some(prev_value) => {
            let diff = new_value - prev_value;
            match is_jitter(diff, jitter_threshold) {
                true => { None }
                false => { Some(new_value) }
            }
        }
    }
}

fn assign_pad_stick_event(coords_state: &mut CoordsState, jitter_threshold: f32, pad_stick_event: PadStickEvent) {
    match pad_stick_event {
        PadStickEvent::FingerLifted => {
            coords_state.reset()
        }
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = discard_jitter(
                coords_state.prev.x, value, jitter_threshold);
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = discard_jitter(
                coords_state.prev.y, value, jitter_threshold);
        }
    }
}

fn writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: Configs,
) -> Result<()> {
    let mut virtual_device = exec_or_eyre!(VirtualDevice::default())?;

    let writing_interval = configs.mouse_interval;
    let is_gaming_mode = configs.buttons_layout.gaming_mode;

    let mut mouse_mode = MouseMode::default();
    let mut pads_coords = PadsCoords::new(&configs.jitter_threshold, &configs.finger_rotation, configs.debug);

    loop {
        let start = Instant::now();

        //MOUSE
        for event in mouse_receiver.try_iter() {
            match event {
                MouseEvent::ModeSwitched => {
                    match mouse_mode {
                        MouseMode::CursorMove => {
                            mouse_mode = MouseMode::Typing;
                        }
                        MouseMode::Typing => {
                            mouse_mode = MouseMode::CursorMove;
                        }
                    }
                }
                MouseEvent::Reset => {
                    mouse_mode = MouseMode::default();
                    pads_coords.reset();
                }
                MouseEvent::LeftPad(pad_stick_event) => {
                    assign_pad_stick_event(&mut pads_coords.left_pad,
                                           configs.jitter_threshold.left_pad,
                                           pad_stick_event)
                }
                MouseEvent::RightPad(pad_stick_event) => {
                    assign_pad_stick_event(&mut pads_coords.right_pad,
                                           configs.jitter_threshold.right_pad,
                                           pad_stick_event)
                }
                MouseEvent::Stick(pad_stick_event) => {
                    assign_pad_stick_event(&mut pads_coords.stick,
                                           configs.jitter_threshold.stick,
                                           pad_stick_event)
                }
            }
        }

        // pads_coords.set_prev_if_cur_is_none();

        if mouse_mode != MouseMode::Typing {
            if pads_coords.right_pad.any_changes() {
                let mouse_diff = pads_coords.right_pad.diff();
                let mouse_diff = mouse_diff.convert(configs.mouse_speed);
                if mouse_diff.is_any_changes() {
                    exec_or_eyre!(virtual_device.move_mouse(mouse_diff.x, -mouse_diff.y))?;
                }
            }
            if !is_gaming_mode {
                if pads_coords.left_pad.any_changes() {
                    let scroll_diff = pads_coords.left_pad.diff();
                    let scroll_diff = scroll_diff.convert(configs.scroll_speed);
                    if scroll_diff.is_any_changes() {
                        exec_or_eyre!(virtual_device.scroll_x(scroll_diff.x))?;
                        exec_or_eyre!(virtual_device.scroll_y(scroll_diff.y))?;
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
                ButtonEvent::Pressed(button) => {
                    exec_or_eyre!(virtual_device.press(button))?
                }
                ButtonEvent::Released(button) => {
                    exec_or_eyre!(virtual_device.release(button))?
                }
            }
        }

        let runtime = start.elapsed();

        if let Some(remaining) = writing_interval.checked_sub(runtime) {
            sleep(remaining);
        }
    }
}

pub fn create_writing_thread(
    mouse_receiver: MouseReceiver,
    button_receiver: ButtonReceiver,
    configs: Configs,
) -> JoinHandle<()> {
    thread::spawn(move || {
        writing_thread(mouse_receiver, button_receiver, configs).unwrap();
    })
}