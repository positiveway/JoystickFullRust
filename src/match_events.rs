use gilrs::{Axis, Button, Event, EventType::*, EventType};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[derive(Display, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum ButtonName {
    BtnUpSideLeft,
    BtnDownSideLeft,
    BtnLeftSideLeft,
    BtnRightSideLeft,
    // 
    BtnUpSideRight,
    BtnDownSideRight,
    BtnLeftSideRight,
    BtnRightSideRight,
    // 
    WingLeft,
    WingRight,
    // 
    LowerTriggerSideLeft,
    UpperTriggerSideLeft,
    LowerTriggerSideRight,
    UpperTriggerSideRight,
    // 
    PadWholeSideLeft,
    PadWholeSideRight,
    //
    PadUpSideLeft,
    PadDownSideLeft,
    PadLeftSideLeft,
    PadRightSideLeft,
    // 
    PadUpSideRight,
    PadDownSideRight,
    PadLeftSideRight,
    PadRightSideRight,
    //
    ExtraBtnSideLeft,
    ExtraBtnSideRight,
    CentralBtn,
}

// Button::South => ButtonName::BtnDownSideRight,
// Button::East => ButtonName::BtnRightSideRight,
// Button::North => ButtonName::BtnUpSideRight,
// Button::West => ButtonName::BtnLeftSideRight,


pub fn match_button(button: &Button) -> &str {
    match button {
        Button::South => "South",
        Button::East => "East",
        Button::North => "North",
        Button::West => "West",
        Button::C => "C",
        Button::Z => "Z",
        Button::LeftTrigger => "LeftTrigger",
        Button::LeftTrigger2 => "LeftTrigger2",
        Button::RightTrigger => "RightTrigger",
        Button::RightTrigger2 => "RightTrigger2",
        Button::Select => "Select",
        Button::Start => "Start",
        Button::Mode => "Mode",
        Button::LeftThumb => "LeftThumb",
        Button::RightThumb => "RightThumb",
        Button::DPadUp => "DPadUp",
        Button::DPadDown => "DPadDown",
        Button::DPadLeft => "DPadLeft",
        Button::DPadRight => "DPadRight",
        Button::Unknown => "Unknown",
    }
}

pub fn match_axis(axis: &Axis) -> &str {
    match axis {
        Axis::LeftStickX => "LeftStickX",
        Axis::LeftStickY => "LeftStickY",
        Axis::LeftZ => "LeftZ",
        Axis::RightStickX => "RightStickX",
        Axis::RightStickY => "RightStickY",
        Axis::RightZ => "RightZ",
        Axis::DPadX => "DPadX",
        Axis::DPadY => "DPadY",
        Axis::Unknown => "Unknown",
    }
}

pub fn match_event(event: &EventType) -> (&str, String, &str, String) {
    let mut button_or_axis = "";
    let mut res_value: f32 = 0.0;
    let mut event_type = "";
    let mut res_code = String::from("");

    match event {
        AxisChanged(axis, value, code) => {
            event_type = "AxisChanged";
            res_value = *value;
            button_or_axis = match_axis(axis);
            res_code = code.to_string()
        }
        ButtonChanged(button, value, code) => {
            event_type = "ButtonChanged";
            res_value = *value;
            button_or_axis = match_button(button);
            res_code = code.to_string()
        }
        ButtonReleased(button, code) => {
            event_type = "ButtonReleased";
            button_or_axis = match_button(button);
            res_code = code.to_string()
        }
        ButtonPressed(button, code) => {
            event_type = "ButtonPressed";
            button_or_axis = match_button(button);
            res_code = code.to_string()
        }
        ButtonRepeated(button, code) => {
            event_type = "ButtonRepeated";
            button_or_axis = match_button(button);
            res_code = code.to_string()
        }
        Connected => {
            event_type = "Connected"
        }
        Disconnected => {
            event_type = "Disconnected"
        }
        Dropped => {
            event_type = "Dropped"
        }
    };
    let res_value = res_value.to_string();
    return (button_or_axis, res_value, event_type, res_code);
}