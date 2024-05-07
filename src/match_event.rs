use color_eyre::eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};

#[derive(PartialOrd, EnumIter, EnumString, AsRefStr, Display, Default, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, )]
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

#[derive(EnumString, AsRefStr, Display, Default, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, )]
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

#[derive(EnumString, AsRefStr, Display, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, )]
// #[strum(serialize_all = "snake_case")]
pub enum EventTypeName {
    AxisChanged,
    ButtonReleased,
    ButtonPressed,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TransformedEvent {
    pub event_type: EventTypeName,
    pub axis: AxisName,
    pub button: ButtonName,
    pub value: f32,
}

pub enum TransformStatus {
    Discarded,
    Unchanged,
    Transformed(TransformedEvent),
    Handled,
}
