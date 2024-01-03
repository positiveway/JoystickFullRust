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

enum TransformationStatus{
    Unchanged,
    Modified(TransformedEvent),
    Discarded,
}

pub fn process_event(event: &EventType, controller_state: &ControllerState) -> Result<()> {
    let event = match_event(event)?;
    if let Some(mut event) = event{
        match transform_triggers(&event) {
            TransformationStatus::Unchanged => {}
            TransformationStatus::Modified(transformed_event) => {
                event = transformed_event;
            }
            TransformationStatus::Discarded => {
                return Ok(())
            }
        };
    }

    Ok(())
}

pub fn transform_left_pad(){

}

pub fn transform_triggers(event: &TransformedEvent) -> TransformationStatus {
    if vec![ButtonName::LowerTriggerAsBtn_SideL, ButtonName::LowerTriggerAsBtn_SideR].contains(&event.button) {
        return TransformationStatus::Discarded
    };
    if vec![ButtonName::LowerTrigger_SideL, ButtonName::LowerTrigger_SideR].contains(&event.button) {
        if event.event_type == EventTypeName::ButtonChanged {
            return TransformationStatus::Modified(
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
            );
        }
    };
    TransformationStatus::Unchanged
}
