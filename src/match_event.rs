use gilrs::{Axis, Button, Event, EventType::*, EventType};
use gilrs::ev::Code;
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use regex::Regex;
use color_eyre::eyre::{bail, Result};
use color_eyre::Report;
use serde::de::Error;
use strum::ParseError;
use crate::configs::Configs;
use crate::exec_or_eyre;


#[derive(EnumIter, EnumString, AsRefStr, Display, Default, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
// #[strum(serialize_all = "snake_case")]
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
    LowerTriggerAsBtn_SideL,
    LowerTriggerAsBtn_SideR,
    //
    UpperTrigger_SideL,
    UpperTrigger_SideR,
    // 
    PadAsBtn_SideL,
    PadAsBtn_SideR,
    StickAsBtn,
    //
    PadAsTouch_SideL,
    PadAsTouch_SideR,
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
    None,
}

impl ButtonName {
    pub fn is_not_init(self) -> bool {
        self == Self::default()
    }

    pub fn bail_if_not_init(self) -> Result<()> {
        if self.is_not_init() {
            bail!("'{self}' button is not specified")
        } else {
            Ok(())
        }
    }

    // pub fn from_config(string: String) -> Result<Self> {
    //     let button_name = Self::try_from(string.as_str());
    //     match button_name {
    //         Ok(button_name) => { Ok(button_name) }
    //         Err(err) => {
    //             Err(Report::new(err).wrap_err(format!("'{string}'")))
    //         }
    //     }
    // }
}

#[derive(EnumString, AsRefStr, Display, Default, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
// #[strum(serialize_all = "snake_case")]
pub enum AxisName {
    PadX_SideL,
    PadY_SideL,
    //
    PadX_SideR,
    PadY_SideR,
    //
    StickX,
    StickY,
    //
    LowerTrigger_SideL,
    LowerTrigger_SideR,
    //
    #[default]
    None,
}

#[derive(EnumString, AsRefStr, Display, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
// #[strum(serialize_all = "snake_case")]
pub enum EventTypeName {
    AxisChanged,
    ButtonReleased,
    ButtonPressed,
    Discarded,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransformedEvent {
    pub event_type: EventTypeName,
    pub axis: AxisName,
    pub button: ButtonName,
    pub value: f32,
}

pub fn match_button(code: u16) -> Result<ButtonName> {
    Ok(match code {
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
        312 => ButtonName::LowerTriggerAsBtn_SideL,
        313 => ButtonName::LowerTriggerAsBtn_SideR,

        310 => ButtonName::UpperTrigger_SideL,
        311 => ButtonName::UpperTrigger_SideR,
        // 
        314 => ButtonName::ExtraBtn_SideL,
        315 => ButtonName::ExtraBtn_SideR,
        316 => ButtonName::ExtraBtnCentral,
        //
        _ => bail!("Unknown button: {code}")
    })
}

pub fn match_axis(code: u16) -> Result<AxisName> {
    Ok(match code {
        16 => AxisName::PadX_SideL,
        17 => AxisName::PadY_SideL,
        //
        3 => AxisName::PadX_SideR,
        4 => AxisName::PadY_SideR,
        //
        21 => AxisName::LowerTrigger_SideL,
        20 => AxisName::LowerTrigger_SideR,
        //
        0 => AxisName::StickX,
        1 => AxisName::StickY,
        //
        _ => bail!("Unknown button: {code}")
    })
}

pub fn match_event(event: &EventType, configs: &Configs) -> Result<TransformedEvent> {
    Ok(match event {
        AxisChanged(axis, value, code) => {
            let code_as_num = print_code(code)?;

            TransformedEvent {
                event_type: EventTypeName::AxisChanged,
                axis: match_axis(code_as_num)?,
                value: *value,
                button: ButtonName::None,
            }
        }
        ButtonChanged(button, value, code) => {
            let code_as_num = print_code(code)?;
            TransformedEvent {
                event_type: match *value {
                    0f32 => EventTypeName::ButtonReleased,
                    1f32 => EventTypeName::ButtonPressed,
                    _ => bail!("Cannot happen"),
                },
                axis: AxisName::None,
                value: *value,
                button: match_button(code_as_num)?,
            }
        }
        Disconnected => {
            TransformedEvent {
                event_type: EventTypeName::ButtonReleased,
                axis: AxisName::None,
                button: configs.buttons_layout.reset_btn,
                value: 0.0,
            }
        }
        _ => {
            TransformedEvent {
                event_type: EventTypeName::Discarded,
                axis: AxisName::None,
                button: ButtonName::None,
                value: 0.0,
            }
        }
    })
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
    let re = exec_or_eyre!(Regex::new(r"\(([0-9]+)\)"))?;
    let binding = code.to_string();
    let Some(caps) = re.captures(binding.as_str()) else {
        bail!("Can't extract code: {}", code.to_string())
    };
    exec_or_eyre!(str::parse::<u16>(&caps[1]))
}

pub fn print_event(event: &EventType) -> Result<String> {
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
    Ok(format!("{event_type}; BtnOrAxis: {button_or_axis}; Value: {:.3}; Code: {code_as_str}; Num: {code_as_num}", res_value))
}