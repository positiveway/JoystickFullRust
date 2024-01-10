use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use mouse_keyboard_input::{Coord, VirtualDevice};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::configs::{Configs, GLOBAL_CONFIGS};
use crate::process_event::{MouseEvent, MouseReceiver, ButtonReceiver, PadStickEvent};

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

    pub fn update(&mut self, new: &Self) {
        self.x = new.x;
        self.y = new.y;
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
    match cur_coord {
        None => { 0f32 }
        Some(cur_value) => {
            match prev_coord {
                None => { 0f32 }
                Some(prev_value) => {
                    cur_value - prev_value
                }
            }
        }
    }
}

fn calc_diff(coords_state: &CoordsState) -> CoordsDiff {
    CoordsDiff {
        x: calc_diff_one_coord(coords_state.prev.x, coords_state.cur.x),
        y: calc_diff_one_coord(coords_state.prev.y, coords_state.cur.y),
    }
}

fn convert_diff(value: f32, multiplier: u16) -> Coord {
    (value * multiplier as f32).round() as Coord
}


#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct CoordsState {
    prev: Coords,
    cur: Coords,
}

impl CoordsState {
    pub fn reset(&mut self) {
        self.prev.reset();
        self.cur.reset();
    }

    pub fn update(&mut self) {
        self.prev.update(&self.cur)
    }
}

#[derive(Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct PadsCoords {
    left_pad: CoordsState,
    right_pad: CoordsState,
    stick: CoordsState,
}

impl PadsCoords {
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
}


fn assign_pad_stick_event(coords_state: &mut CoordsState, pad_stick_event: PadStickEvent) {
    match pad_stick_event {
        PadStickEvent::FingerLifted => {
            coords_state.cur.reset()
        }
        PadStickEvent::MovedX(value) => {
            coords_state.cur.x = Some(value);
        }
        PadStickEvent::MovedY(value) => {
            coords_state.cur.y = Some(value);
        }
    }
}

pub fn create_writing_thread(mouse_receiver: MouseReceiver, button_receiver: ButtonReceiver) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut virtual_device = VirtualDevice::default().unwrap();

        let configs: Configs = GLOBAL_CONFIGS.clone();
        let writing_interval = configs.mouse_interval;
        let is_gaming_mode = configs.buttons_layout.gaming_mode;

        let mut mouse_mode = MouseMode::default();
        let mut pads_coords = PadsCoords::default();


        let mut mouse_func = || {
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
                let mouse_diff = calc_diff(&pads_coords.right_pad);
                let mouse_diff = mouse_diff.convert(configs.mouse_speed);
                if mouse_diff.is_any_changes() {
                    virtual_device.move_mouse(mouse_diff.x, -mouse_diff.y).unwrap();
                }

                if !is_gaming_mode {
                    let scroll_diff = calc_diff(&pads_coords.left_pad);
                    if scroll_diff.y.abs() > configs.horizontal_threshold_f32 {
                        let scroll_diff = scroll_diff.convert(configs.scroll_speed);
                        virtual_device.scroll_y(scroll_diff.y).unwrap();
                    } else {
                        let scroll_diff = scroll_diff.convert(configs.scroll_speed);
                        virtual_device.scroll_y(scroll_diff.x).unwrap();
                    }
                }
            }

            pads_coords.update();
        };

        let mut button_func = || {
            for event in button_receiver.try_iter() {}
        };

        loop {
            let start = Instant::now();

            mouse_func();
            button_func();

            let runtime = start.elapsed();

            if let Some(remaining) = writing_interval.checked_sub(runtime) {
                sleep(remaining);
            }
        }
    })

    // scheduler.join().expect("Scheduler panicked");
}