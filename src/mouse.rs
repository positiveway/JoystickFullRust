use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use mouse_keyboard_input::{Coord, VirtualDevice};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::configs::{Configs, JitterThreshold};
use color_eyre::eyre::{Result};
use crate::process_event::{MouseEvent, MouseReceiver, ButtonReceiver, PadStickEvent};
use crate::exec_or_eyre;

use std::f64::consts::PI;

fn smoothing_factor(t_e: f64, cutoff: f64) -> f64 {
    let r = 2.0 * PI * cutoff * t_e;
    r / (r + 1.0)
}

fn exponential_smoothing(a: f64, x: f64, x_prev: f64) -> f64 {
    a * x + (1.0 - a) * x_prev
}

fn create_filter(cutoff: f64, beta: f64) -> impl FnMut(f64, f64) -> f64 {
    let mut self_filter = Filter {
        cutoff: cutoff,
        beta: beta,
        d_cutoff: 1.0,
        dx0: 0.0,
        x_prev: 0.0,
        dx_prev: 0.0,
        t_prev: 0.0,
    };

    move |t: f64, x: f64| {
        let t_e = t - self_filter.t_prev;
        let a_d = smoothing_factor(t_e, self_filter.d_cutoff);
        let dx = (x - self_filter.x_prev) / t_e;
        let dx_hat = exponential_smoothing(a_d, dx, self_filter.dx_prev);
        let cutoff = self_filter.cutoff + self_filter.beta * dx_hat.abs();
        let a = smoothing_factor(t_e, cutoff);
        let x_hat = exponential_smoothing(a, x, self_filter.x_prev);
        self_filter.x_prev = x_hat;
        self_filter.dx_prev = dx_hat;
        self_filter.t_prev = t;
        x_hat
    }
}

struct Filter {
    cutoff: f64,
    beta: f64,
    d_cutoff: f64,
    dx0: f64,
    x_prev: f64,
    dx_prev: f64,
    t_prev: f64,
}

// fn main() {
//     let filter = create_filter(1.0, 0.0);
//     let result = filter(0.0, 0.0);
//     println!("{}", result);
// }

pub fn hypot<T>(a: T, b: T) -> f64
    where T: core::ops::Mul<T, Output=T> + core::ops::Add<T, Output=T> + core::convert::Into<f64> + Copy
{
    (a * a + b * b).into().sqrt()
}

#[derive(Display, Eq, Hash, PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MouseMode {
    #[default]
    CursorMove,
    Typing,
}

#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
struct Coords {
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

fn discard_jitter(value: f32, jitter_threshold: f32) -> f32 {
    if value.abs() <= jitter_threshold {
        0.0
    } else {
        value
    }
}

fn calc_diff_one_coord(prev_coord: Option<f32>, cur_coord: Option<f32>, jitter_threshold: f32) -> f32 {
    match cur_coord {
        None => { 0f32 }
        Some(cur_value) => {
            match prev_coord {
                None => { 0f32 }
                Some(prev_value) => {
                    let diff = cur_value - prev_value;
                    discard_jitter(diff, jitter_threshold)
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
}

impl CoordsState {
    pub fn new(jitter_threshold: f32) -> Self {
        Self {
            prev: Default::default(),
            cur: Default::default(),
            jitter_threshold,
        }
    }

    pub fn reset(&mut self) {
        self.prev.reset();
        self.cur.reset();
    }

    pub fn update(&mut self) {
        self.prev.update(&self.cur)
    }

    pub fn update_if_not_init(&mut self) {
        self.prev.update_if_not_init(&self.cur);
    }

    pub fn diff(&self) -> CoordsDiff {
        CoordsDiff {
            x: calc_diff_one_coord(self.prev.x, self.cur.x, self.jitter_threshold),
            y: calc_diff_one_coord(self.prev.y, self.cur.y, self.jitter_threshold),
        }
    }

    pub fn convert_diff(&self, multiplier: u16) -> ConvertedCoordsDiff {
        self.diff().convert(multiplier)
    }

    pub fn diff_and_update(&mut self) -> CoordsDiff {
        let diff = self.diff();
        if diff.is_any_changes() {
            self.update();
        }
        diff
    }

    pub fn convert_diff_and_update(&mut self, multiplier: u16) -> ConvertedCoordsDiff {
        let converted_diff = self.convert_diff(multiplier);
        if converted_diff.is_any_changes() {
            self.update();
        }
        converted_diff
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PadsCoords {
    left_pad: CoordsState,
    right_pad: CoordsState,
    stick: CoordsState,
}

impl PadsCoords {
    pub fn new(jitter_threshold: &JitterThreshold) -> Self {
        Self {
            left_pad: CoordsState::new(jitter_threshold.left_pad),
            right_pad: CoordsState::new(jitter_threshold.right_pad),
            stick: CoordsState::new(jitter_threshold.stick),
        }
    }

    pub fn reset(&mut self) {
        self.left_pad.reset();
        self.right_pad.reset();
        self.stick.reset();
    }

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

fn assign_pad_stick_event(coords_state: &mut CoordsState, pad_stick_event: PadStickEvent) {
    match pad_stick_event {
        PadStickEvent::FingerLifted => {
            coords_state.reset()
        }
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = Some(value);
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = Some(value);
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
    let mut pads_coords = PadsCoords::new(&configs.jitter_threshold);

    let mut mouse_func = || -> Result<()> {
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
                    assign_pad_stick_event(&mut pads_coords.left_pad, pad_stick_event)
                }
                MouseEvent::RightPad(pad_stick_event) => {
                    assign_pad_stick_event(&mut pads_coords.right_pad, pad_stick_event)
                }
                MouseEvent::Stick(pad_stick_event) => {
                    assign_pad_stick_event(&mut pads_coords.stick, pad_stick_event)
                }
            }
        }

        if mouse_mode != MouseMode::Typing {
            let mouse_diff = pads_coords.right_pad.diff_and_update();
            let mouse_diff = mouse_diff.convert(configs.mouse_speed);
            if mouse_diff.is_any_changes() {
                exec_or_eyre!(virtual_device.move_mouse(mouse_diff.x, -mouse_diff.y))?;
            }

            if !is_gaming_mode {
                let scroll_diff = pads_coords.left_pad.diff_and_update();
                let scroll_diff = scroll_diff.convert(configs.scroll_speed);
                if scroll_diff.is_any_changes() {
                    exec_or_eyre!(virtual_device.scroll_x(scroll_diff.x))?;
                    exec_or_eyre!(virtual_device.scroll_y(scroll_diff.y))?;
                }
            }
        }

        pads_coords.stick.update();
        //Important to keep
        pads_coords.update_if_not_init();
        // pads_coords.update();
        Ok(())
    };

    let mut button_func = || -> Result<()> {
        for event in button_receiver.try_iter() {}
        Ok(())
    };

    loop {
        let start = Instant::now();

        mouse_func()?;
        button_func()?;

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

    // scheduler.join().expect("Scheduler panicked");
}