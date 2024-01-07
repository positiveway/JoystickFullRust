use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::Instant;
use mouse_keyboard_input::{Coord, VirtualDevice};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::configs::{Configs, GLOBAL_CONFIGS};
use crate::process_event::{PadEvent, MouseReceiver, ButtonReceiver};

#[derive(Display, Eq, Hash, PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MouseMode {
    #[default]
    CursorMove,
    Scrolling,
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
struct Coords {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
struct CoordsDiff {
    pub x: f32,
    pub y: f32,
}


impl Coords {
    pub fn default() -> Self {
        Self { x: None, y: None }
    }

    pub fn reset(&mut self) {
        self.x = None;
        self.y = None;
    }
}

fn calc_diff(prev_coord: Option<f32>, cur_coord: Option<f32>) -> f32 {
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

fn convert_diff(value: f32, multiplier: u16) -> Coord {
    (value * multiplier as f32).round() as Coord
}

pub fn create_writing_thread(mouse_receiver: MouseReceiver, button_receiver: ButtonReceiver) -> JoinHandle<()> {
    thread::spawn(move || {
        let mut virtual_device = VirtualDevice::default().unwrap();

        let configs: Configs = GLOBAL_CONFIGS.clone();
        let writing_interval = configs.mouse_interval;

        let mut mouse_mode = MouseMode::default();
        let mut cur_coords = Coords::default();
        let mut prev_coords = Coords::default();

        let mut mouse_func = || {
            for event in mouse_receiver.try_iter() {
                match event {
                    PadEvent::Reset => {
                        cur_coords.reset();
                        mouse_mode = MouseMode::default();
                    }

                    PadEvent::FingerLifted => {
                        cur_coords.reset();
                    }
                    PadEvent::Moved(value, is_x) => {
                        match is_x {
                            true => {
                                cur_coords.x = Some(value);
                            }
                            false => {
                                cur_coords.y = Some(value);
                            }
                        }
                    }
                    PadEvent::ModeSwitched => {
                        match mouse_mode {
                            MouseMode::CursorMove => {
                                mouse_mode = MouseMode::Scrolling;
                            }
                            MouseMode::Scrolling => {
                                mouse_mode = MouseMode::CursorMove;
                            }
                        }
                    }
                }
            }

            let multiplier = match mouse_mode {
                MouseMode::CursorMove => { configs.mouse_speed }
                MouseMode::Scrolling => { configs.scroll_speed }
            };

            let coords_diff = CoordsDiff {
                x: calc_diff(prev_coords.x, cur_coords.x),
                y: calc_diff(prev_coords.y, cur_coords.y),
            };

            match mouse_mode {
                MouseMode::CursorMove => {
                    virtual_device.move_mouse(
                        convert_diff(coords_diff.x, multiplier),
                        convert_diff(coords_diff.y, multiplier),
                    ).unwrap();
                }
                MouseMode::Scrolling => {
                    if coords_diff.y.abs() > configs.horizontal_threshold_f32 {
                        virtual_device.scroll_y(convert_diff(coords_diff.y, multiplier)).unwrap();
                    } else {
                        virtual_device.scroll_x(convert_diff(coords_diff.x, multiplier)).unwrap();
                    }
                }
            }

            prev_coords.x = cur_coords.x;
            prev_coords.y = cur_coords.y;
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