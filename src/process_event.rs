use std::collections::HashMap;
use gilrs::EventType;
use color_eyre::eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use crate::match_event::*;
use crate::config::GLOBAL_CONFIG;

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct ControllerState {
    pub pressed_buttons: HashMap<ButtonName, bool>,
}

pub fn process_event(event: &EventType, controller_state: &ControllerState) -> Result<()> {
    let event = match_event(event)?;
    if let Some(mut event) = event{
        if let Some(transformed_event) = transform_triggers(&event){
            event = transformed_event
        };

    }

    Ok(())
}

pub fn transform_triggers(event: &TransformedEvent) -> Option<TransformedEvent> {
    if vec![ButtonName::LowerTrigger_SideL, ButtonName::LowerTrigger_SideR].contains(&event.button) {
        if event.event_type == EventTypeName::ButtonChanged {
            return Some(
                if event.value > GLOBAL_CONFIG.triggers_threshold_pct as f32 / 100.0 {
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
    None
}
