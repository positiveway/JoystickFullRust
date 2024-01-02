use gilrs::{Axis, Button, Event, EventType::*, EventType};
use gilrs::ev::Code;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use regex::Regex;
use color_eyre::eyre::{bail, Result};
use crate::exec_or_eyre;


#[derive(Display, Eq, Hash, PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum ButtonName {
    BtnUp_SideL,
    BtnDown_SideL,
    BtnLeft_SideL,
    BtnRight_SideL,
    // 
    BtnUp_SideR,
    BtnDown_SideR,
    BtnLeft_SideR,
    BtnRight_SideR,
    // 
    Wing_SideL,
    Wing_SideR,
    // 
    LowerTrigger_SideL,
    UpperTrigger_SideL,
    LowerTrigger_SideR,
    UpperTrigger_SideR,
    // 
    PadAsBtn_SideL,
    PadAsBtn_SideR,
    PadAsTouch_SideL,
    PadAsTouch_SideR,
    StickAsBtn,
    //
    PadUp_SideL,
    PadDown_SideL,
    PadLeft_SideL,
    PadRight_SideL,
    // 
    PadUp_SideR,
    PadDown_SideR,
    PadLeft_SideR,
    PadRight_SideR,
    //
    ExtraBtn_SideL,
    ExtraBtn_SideR,
    ExtraBtnCentral,
    //
    #[default]
    Unknown,
}

#[derive(Display, Eq, Hash, PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum AxisName {
    PadX_SideL,
    PadY_SideL,
    PadX_SideR,
    PadY_SideR,
    StickX,
    StickY,
    //
    #[default]
    Unknown,
}


pub fn match_button(code: u16) -> Result<ButtonName> {
    let res = match code {
        304 => ButtonName::BtnDown_SideR,
        305 => ButtonName::BtnRight_SideR,
        308 => ButtonName::BtnUp_SideR,
        307 => ButtonName::BtnLeft_SideR,
        //
        336 => ButtonName::Wing_SideL,
        337 => ButtonName::Wing_SideR,
        //
        289 => ButtonName::PadAsTouch_SideL,
        290 => ButtonName::PadAsTouch_SideR,
        318 => ButtonName::PadAsBtn_SideR,
        317 => ButtonName::StickAsBtn,
        //
        545 => ButtonName::PadDown_SideL,
        547 => ButtonName::PadRight_SideL,
        544 => ButtonName::PadUp_SideL,
        546 => ButtonName::PadLeft_SideL,
        //
        312 => ButtonName::LowerTrigger_SideL,
        310 => ButtonName::UpperTrigger_SideL,
        313 => ButtonName::LowerTrigger_SideR,
        311 => ButtonName::UpperTrigger_SideR,
        // 
        314 => ButtonName::ExtraBtn_SideL,
        315 => ButtonName::ExtraBtn_SideR,
        316 => ButtonName::ExtraBtnCentral,
        //
        _ => ButtonName::Unknown,
    };
    if res == ButtonName::Unknown {
        bail!("Unknown button: {code}")
    }
    Ok(res)
}

pub fn match_axis(code: u16) -> Result<AxisName> {
    let res = match code {
        16 => AxisName::PadX_SideL,
        17 => AxisName::PadY_SideL,
        //
        3 => AxisName::PadX_SideR,
        4 => AxisName::PadY_SideR,
        //
        0 => AxisName::StickX,
        1 => AxisName::StickY,
        //
        _ => AxisName::Unknown,
    };
    if res == AxisName::Unknown {
        bail!("Unknown button: {code}")
    }
    Ok(res)
}

pub fn print_button(button: &Button) -> &str {
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

pub fn print_axis(axis: &Axis) -> &str {
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

fn print_code(code: &Code) -> Result<u16> {
    let re = Regex::new(r"\(([0-9]+)\)").unwrap();
    let binding = code.to_string();
    let Some(caps) = re.captures(binding.as_str()) else {
        bail!("Can't extract code: {}", code.to_string())
    };
    exec_or_eyre!(str::parse::<u16>(&caps[1]))
}

pub fn print_event(event: &EventType) -> Result<(&str, String, &str, String, u16)> {
    let mut button_or_axis = "";
    let mut res_value: f32 = 0.0;
    let mut event_type = "";
    let mut code_as_str = String::from("");
    let mut code_as_num: u16 = 0;

    match event {
        AxisChanged(axis, value, code) => {
            event_type = "AxisChanged";
            res_value = *value;
            button_or_axis = print_axis(axis);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonChanged(button, value, code) => {
            event_type = "ButtonChanged";
            res_value = *value;
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonReleased(button, code) => {
            event_type = "ButtonReleased";
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonPressed(button, code) => {
            event_type = "ButtonPressed";
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonRepeated(button, code) => {
            event_type = "ButtonRepeated";
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
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
    let mut res_value = res_value.to_string();
    const MAX_LENGTH: usize = 4;
    if res_value.len() > MAX_LENGTH {
        res_value = res_value[..MAX_LENGTH].parse()?
    }
    Ok((button_or_axis, res_value, event_type, code_as_str, code_as_num))
}