use std::collections::HashMap;
use gilrs::EventType;
use color_eyre::eyre::{bail, OptionExt, Result};
use serde::{Deserialize, Serialize};
use crate::match_event::*;
use crate::configs::{Configs};
use crossbeam_channel::{Sender, Receiver, bounded};
use mouse_keyboard_input::Button;
use strum_macros::Display;
use crate::exec_or_eyre;
use crate::process_event::PadStickEvent::FingerLifted;


#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PadStickEvent {
    FingerLifted,
    MovedX(f32),
    MovedY(f32),
}

#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MouseEvent {
    LeftPad(PadStickEvent),
    RightPad(PadStickEvent),
    Stick(PadStickEvent),
    ModeSwitched,
    Reset,
}

pub type MouseSender = Sender<MouseEvent>;
pub type MouseReceiver = Receiver<MouseEvent>;

pub type ButtonSender = Sender<Button>;
pub type ButtonReceiver = Receiver<Button>;

#[derive(Clone, Debug)]
pub struct ControllerState {
    pub pressed_buttons: HashMap<ButtonName, bool>,
    //
    pub mouse_sender: MouseSender,
    pub mouse_receiver: MouseReceiver,
    pub button_sender: ButtonSender,
    pub button_receiver: ButtonReceiver,
    //
    pub SWITCH_BUTTON: ButtonName,
    pub RESET_BUTTON: ButtonName,
    pub configs: Configs,
}

impl ControllerState {
    pub fn new(configs: Configs) -> Self {
        let (mouse_sender, mouse_receiver) = bounded(configs.channel_size);
        let (button_sender, button_receiver) = bounded(configs.channel_size);
        Self {
            pressed_buttons: Default::default(),
            mouse_sender,
            mouse_receiver,
            button_sender,
            button_receiver,
            SWITCH_BUTTON: configs.buttons_layout.switch_button,
            RESET_BUTTON: configs.buttons_layout.reset_button,
            configs,
        }
    }
}


enum TransformStatus {
    Discarded,
    Unchanged,
    Transformed(TransformedEvent),
    Handled,
}

pub fn process_event(event: &EventType, controller_state: &ControllerState) -> Result<()> {
    let mut event = match_event(event, &controller_state.configs)?;
    if event.event_type == EventTypeName::Discarded {
        return Ok(());
    }

    match transform_triggers(&event, &controller_state.configs) {
        TransformStatus::Discarded | TransformStatus::Handled => {
            return Ok(());
        }
        TransformStatus::Transformed(transformed_event) => {
            event = transformed_event;
        }
        TransformStatus::Unchanged => {}
    };

    match transform_left_pad(&event) {
        TransformStatus::Discarded | TransformStatus::Handled => {
            return Ok(());
        }
        TransformStatus::Transformed(transformed_event) => {
            event = transformed_event;
        }
        TransformStatus::Unchanged => {}
    };

    match process_pad_stick(&event, controller_state)? {
        TransformStatus::Discarded | TransformStatus::Handled => {
            return Ok(());
        }
        TransformStatus::Transformed(transformed_event) => {
            event = transformed_event;
        }
        TransformStatus::Unchanged => {}
    };

    Ok(())
}


pub fn process_pad_stick(event: &TransformedEvent, controller_state: &ControllerState) -> Result<TransformStatus> {
    let send_mouse_event = |mouse_event: MouseEvent| -> Result<()> {
        exec_or_eyre!(controller_state.mouse_sender.send(mouse_event))
    };

    match event.event_type {
        EventTypeName::ButtonReleased => {
            match event.button {
                ButtonName::PadAsTouch_SideR => {
                    send_mouse_event(MouseEvent::RightPad(FingerLifted))?;
                    return Ok(TransformStatus::Handled);
                }
                ButtonName::PadAsTouch_SideL => {
                    send_mouse_event(MouseEvent::LeftPad(FingerLifted))?;
                    return Ok(TransformStatus::Handled);
                }
                _ => {}
            };

            if event.button == controller_state.RESET_BUTTON {
                send_mouse_event(MouseEvent::Reset)?;
                return Ok(TransformStatus::Unchanged);
            } else if event.button == controller_state.SWITCH_BUTTON {
                send_mouse_event(MouseEvent::ModeSwitched)?;
                return Ok(TransformStatus::Handled);
            };
        }
        _ => {}
    }

    match event.event_type {
        EventTypeName::AxisChanged => {
            if event.value == 0f32 {
                return Ok(TransformStatus::Discarded);
            };

            match event.axis {
                AxisName::PadX_SideL => {
                    send_mouse_event(MouseEvent::LeftPad(PadStickEvent::MovedX(event.value)))?;
                    return Ok(TransformStatus::Handled);
                }
                AxisName::PadY_SideL => {
                    send_mouse_event(MouseEvent::LeftPad(PadStickEvent::MovedY(event.value)))?;
                    return Ok(TransformStatus::Handled);
                }
                AxisName::PadX_SideR => {
                    send_mouse_event(MouseEvent::RightPad(PadStickEvent::MovedX(event.value)))?;
                    return Ok(TransformStatus::Handled);
                }
                AxisName::PadY_SideR => {
                    send_mouse_event(MouseEvent::RightPad(PadStickEvent::MovedY(event.value)))?;
                    return Ok(TransformStatus::Handled);
                }
                AxisName::StickX => {
                    send_mouse_event(MouseEvent::Stick(PadStickEvent::MovedX(event.value)))?;
                    return Ok(TransformStatus::Handled);
                }
                AxisName::StickY => {
                    send_mouse_event(MouseEvent::Stick(PadStickEvent::MovedY(event.value)))?;
                    return Ok(TransformStatus::Handled);
                }
                _ => {}
            };
        }
        _ => {}
    };

    Ok(TransformStatus::Unchanged)
}


pub fn transform_left_pad(event: &TransformedEvent) -> TransformStatus {
    match event.button {
        ButtonName::PadDown_SideL |
        ButtonName::PadRight_SideL |
        ButtonName::PadUp_SideL |
        ButtonName::PadLeft_SideL => {
            TransformStatus::Transformed(TransformedEvent {
                event_type: event.event_type,
                axis: AxisName::None,
                value: event.value,
                button: ButtonName::PadAsBtn_SideL,
            })
        }
        _ => TransformStatus::Unchanged
    }
}

pub fn transform_triggers(event: &TransformedEvent, configs: &Configs) -> TransformStatus {
    match event.button {
        ButtonName::LowerTriggerAsBtn_SideL | ButtonName::LowerTriggerAsBtn_SideR => {
            return TransformStatus::Discarded;
        }
        _ => {}
    };

    match event.axis {
        AxisName::LowerTrigger_SideL | AxisName::LowerTrigger_SideR => {
            let button = match event.axis {
                AxisName::LowerTrigger_SideL => { ButtonName::LowerTriggerAsBtn_SideL }
                AxisName::LowerTrigger_SideR => { ButtonName::LowerTriggerAsBtn_SideR }
                _ => { ButtonName::None }
            };
            return TransformStatus::Transformed({
                if event.value > configs.triggers_threshold_f32 {
                    TransformedEvent {
                        event_type: EventTypeName::ButtonPressed,
                        axis: AxisName::None,
                        value: 1f32,
                        button,
                    }
                } else {
                    TransformedEvent {
                        event_type: EventTypeName::ButtonReleased,
                        axis: AxisName::None,
                        value: 0f32,
                        button,
                    }
                }
            });
        }
        _ => {}
    };

    TransformStatus::Unchanged
}
