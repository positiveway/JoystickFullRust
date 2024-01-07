use std::collections::HashMap;
use gilrs::EventType;
use color_eyre::eyre::{bail, OptionExt, Result};
use serde::{Deserialize, Serialize};
use crate::match_event::*;
use crate::configs::GLOBAL_CONFIGS;
use crossbeam_channel::{Sender, Receiver, bounded};
use mouse_keyboard_input::Button;
use strum_macros::Display;
use crate::exec_or_eyre;


#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PadEvent {
    FingerLifted,
    // value, is_x
    Moved(f32, bool),
    ModeSwitched,
    Reset,
}

pub type MouseSender = Sender<PadEvent>;
pub type MouseReceiver = Receiver<PadEvent>;

pub type ButtonSender = Sender<Button>;
pub type ButtonReceiver = Receiver<Button>;

#[derive(Clone, Debug)]
pub struct ControllerState {
    pub pressed_buttons: HashMap<ButtonName, bool>,
    pub mouse_sender: MouseSender,
    pub mouse_receiver: MouseReceiver,
    pub button_sender: ButtonSender,
    pub button_receiver: ButtonReceiver,
}

impl ControllerState {
    pub fn default() -> Self {
        let (mouse_sender, mouse_receiver) = bounded(GLOBAL_CONFIGS.channel_size);
        let (button_sender, button_receiver) = bounded(GLOBAL_CONFIGS.channel_size);
        Self {
            pressed_buttons: Default::default(),
            mouse_sender,
            mouse_receiver,
            button_sender,
            button_receiver,
        }
    }
}

pub fn process_event(event: &EventType, controller_state: &ControllerState) -> Result<()> {
    let mut event = match_event(event)?;
    event = transform_triggers(event);
    event = transform_left_pad(event);

    if event.event_type == EventTypeName::Discarded {
        return Ok(());
    }

    if let _handled = process_right_pad(&event, controller_state) {
        return Ok(());
    }


    Ok(())
}

pub fn process_right_pad(event: &TransformedEvent, controller_state: &ControllerState) -> Result<bool> {
    let switch_button: ButtonName = GLOBAL_CONFIGS.buttons_layout.switch_button;
    let reset_button: ButtonName = GLOBAL_CONFIGS.buttons_layout.reset_button;

    let pad_event: Option<PadEvent> = {
        match event.button {
            ButtonName::PadAsTouch_SideR => {
                match event.event_type {
                    EventTypeName::ButtonReleased => {
                        Some(PadEvent::FingerLifted)
                    }
                    _ => { None }
                }
            }
            _ => { None }
        };

        if event.button == switch_button {
            match event.event_type {
                EventTypeName::ButtonReleased => {
                    Some(PadEvent::ModeSwitched)
                }
                _ => { None }
            }
        } else if event.button == reset_button {
            match event.event_type {
                EventTypeName::ButtonReleased => {
                    Some(PadEvent::Reset)
                }
                _ => { None }
            }
        } else {
            None
        };

        // match event.event_type {
        //     EventTypeName::AxisChanged => {}
        //     EventTypeName::ButtonReleased => {}
        //     EventTypeName::ButtonPressed => {}
        //     EventTypeName::ButtonChanged => {}
        //     EventTypeName::Unknown => {}
        //     EventTypeName::Discarded => {}
        // }
        match event.axis {
            AxisName::PadX_SideR => {
                Some(PadEvent::Moved(event.value, true))
            }
            AxisName::PadY_SideR => {
                Some(PadEvent::Moved(event.value, false))
            }
            _ => { None }
        };

        None
    };

    match pad_event {
        None => {
            Ok(false)
        }
        Some(pad_event) => {
            exec_or_eyre!(controller_state.mouse_sender.send(pad_event))?;
            Ok(true)
        }
    }
}

pub fn transform_left_pad(event: TransformedEvent) -> TransformedEvent {
    match event.button {
        ButtonName::PadDown_SideL |
        ButtonName::PadRight_SideL |
        ButtonName::PadUp_SideL |
        ButtonName::PadLeft_SideL => {
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
