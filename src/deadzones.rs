use gilrs::{Axis, Gamepad, Gilrs};
use color_eyre::eyre::{OptionExt, Result};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
struct Coords {
    pub x: f32,
    pub y: f32,
}

fn get_deadzone(gamepad: &Gamepad, axis: Axis) -> Result<f32> {
    gamepad.deadzone(gamepad.axis_code(axis).ok_or_eyre("No such axis")?).ok_or_eyre("Can't get a deadzone")
}

fn get_gamepad(gilrs: &Gilrs, id: usize) -> Result<Gamepad> {
    let mut res: Option<Gamepad> = None;
    for (_id, gamepad) in gilrs.gamepads() {
        let _id: usize = _id.into();
        if _id == id {
            res = Some(gamepad);
        }
    };
    res.ok_or_eyre("Couldn't get Gamepad by id")
}

pub fn print_deadzones(gilrs: &Gilrs, id: usize) -> Result<()> {
    let gamepad0 = get_gamepad(gilrs, id)?;
    let mut deadzone = Coords::default();

    deadzone.x = get_deadzone(&gamepad0, Axis::LeftStickX)?;
    deadzone.y = get_deadzone(&gamepad0, Axis::LeftStickY)?;
    println!("Left joystick deadzones: ({}, {})", deadzone.x, deadzone.y);

    deadzone.x = get_deadzone(&gamepad0, Axis::RightStickX)?;
    deadzone.y = get_deadzone(&gamepad0, Axis::RightStickY)?;
    println!("Right joystick deadzones: ({}, {})", deadzone.x, deadzone.y);
    Ok(())
}