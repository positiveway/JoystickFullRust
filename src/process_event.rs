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
    Modified,
    Discarded,
}

pub fn process_event(event: &EventType, controller_state: &ControllerState) -> Result<()> {
    let event = match_event(event)?;
    if let Some(mut event) = event{
        if let (status, transformed_event) = transform_triggers(&event){
            match status {
                TransformationStatus::Unchanged => {}
                TransformationStatus::Modified => {
                    event = transformed_event.ok_or_eyre("Triggers transform error")?;
                }
                TransformationStatus::Discarded => {
                    return Ok(())
                }
            }
        };

    }

    Ok(())
}

pub fn transform_triggers(event: &TransformedEvent) -> (TransformationStatus, Option<TransformedEvent>) {
    if vec![ButtonName::LowerTriggerAsBtn_SideL, ButtonName::LowerTriggerAsBtn_SideR].contains(&event.button) {
        return (TransformationStatus::Discarded, None)
    };
    if vec![ButtonName::LowerTrigger_SideL, ButtonName::LowerTrigger_SideR].contains(&event.button) {
        if event.event_type == EventTypeName::ButtonChanged {
            return (TransformationStatus::Modified, Some(
                if event.value > GLOBAL_CONFIGS.triggers_threshold_pct as f32 / 100.0 {
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
            ));
        }
    };
    (TransformationStatus::Unchanged, None)
}
