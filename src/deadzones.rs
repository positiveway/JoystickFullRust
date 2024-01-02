use gilrs::{Axis, Gamepad, Gilrs};

#[derive(Clone, Copy, Debug, Default)]
pub struct Coords {
    pub x: f32,
    pub y: f32,
}

fn get_deadzone(gamepad: &Gamepad, axis: Axis) -> f32 {
    gamepad.deadzone(gamepad.axis_code(axis).unwrap()).unwrap()
}

fn get_gamepad(gilrs: &Gilrs, id: usize) -> Gamepad {
    let mut res: Option<Gamepad> = None;
    for (_id, gamepad) in gilrs.gamepads() {
        let _id: usize = _id.into();
        if _id == id {
            res = Some(gamepad);
        }
    };
    res.unwrap()
}

pub fn print_deadzones(gilrs: &Gilrs, id: usize) {
    let gamepad0 = get_gamepad(gilrs, id);
    let mut deadzone = Coords::default();

    deadzone.x = get_deadzone(&gamepad0, Axis::LeftStickX);
    deadzone.y = get_deadzone(&gamepad0, Axis::LeftStickY);
    println!("Left joystick deadzones: ({}, {})", deadzone.x, deadzone.y);

    deadzone.x = get_deadzone(&gamepad0, Axis::RightStickX);
    deadzone.y = get_deadzone(&gamepad0, Axis::RightStickY);
    println!("Right joystick deadzones: ({}, {})", deadzone.x, deadzone.y);
}