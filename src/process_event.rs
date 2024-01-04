use std::collections::HashMap;
use gilrs::EventType;
use color_eyre::eyre::{bail, OptionExt, Result};
use serde::{Deserialize, Serialize};
use crate::match_event::*;
use crate::configs::GLOBAL_CONFIGS;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ControllerState {
    pub pressed_buttons: HashMap<ButtonName, bool>,
}

enum TransformationStatus {
    Unchanged,
    Modified(TransformedEvent),
    Discarded,
}

pub fn process_event(event: &EventType, controller_state: &ControllerState) -> Result<()> {
    let mut event = match_event(event)?;
    event = transform_triggers(event);
    event = transform_left_pad(event);

    Ok(())
}

pub fn transform_left_pad(event: TransformedEvent) -> TransformedEvent {
    match event.button {
        ButtonName::PadDown_SideL | ButtonName::PadRight_SideL | ButtonName::PadUp_SideL | ButtonName::PadLeft_SideL => {
            TransformedEvent {
                event_type: event.event_type,
                axis: Default::default(),
                value: event.value,
                button: ButtonName::PadAsBtn_SideL,
            }
        }
        _ => TransformedEvent::discarded()
    }
}

pub fn transform_triggers(event: TransformedEvent) -> TransformedEvent {
    match event.button {
        ButtonName::LowerTriggerAsBtn_SideL | ButtonName::LowerTriggerAsBtn_SideR => {
            TransformedEvent::discarded()
        }
        ButtonName::LowerTrigger_SideL | ButtonName::LowerTrigger_SideR => {
            // this includes all buttons events so values 1.0 and 0.0 are handled
            // EventTypeName::ButtonReleased | EventTypeName::ButtonPressed | EventTypeName::ButtonChanged => {
            if event.value > GLOBAL_CONFIGS.triggers_threshold_f32 {
                TransformedEvent {
                    event_type: EventTypeName::ButtonPressed,
                    axis: Default::default(),
                    value: 1f32,
                    button: event.button,
                }
            } else {
                TransformedEvent {
                    event_type: EventTypeName::ButtonReleased,
                    axis: Default::default(),
                    value: 0f32,
                    button: event.button,
                }
            }
        }
        _ => event
    }
}
