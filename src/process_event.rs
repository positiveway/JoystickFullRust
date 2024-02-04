use std::collections::HashMap;
use gilrs::EventType;
use color_eyre::eyre::{bail, OptionExt, Result};
use serde::{Deserialize, Serialize};
use crate::match_event::*;
use crate::configs::{Configs};
use crossbeam_channel::{Sender, Receiver, bounded};
use lazy_static::lazy_static;
use mouse_keyboard_input::Button;
use strum_macros::{Display};
use crate::exec_or_eyre;
use crate::process_event::PadStickEvent::FingerLifted;
use crate::math_ops::RangeConverterBuilder;


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
    pub RESET_BTN: ButtonName,
    pub SWITCH_MODE_BTN: ButtonName,
    //
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
            RESET_BTN: configs.buttons_layout.reset_btn,
            SWITCH_MODE_BTN: configs.buttons_layout.switch_mode_btn,
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

    match transform_triggers(&mut event, &controller_state.configs) {
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
            if let Some(event_to_send) =
                match event.button {
                    ButtonName::PadAsTouch_SideR => {
                        Some(MouseEvent::RightPad(FingerLifted))
                    }
                    ButtonName::PadAsTouch_SideL => {
                        Some(MouseEvent::LeftPad(FingerLifted))
                    }
                    _ => {
                        None
                    }
                }
            {
                send_mouse_event(event_to_send)?;
                return Ok(TransformStatus::Handled);
            };

            if event.button == controller_state.RESET_BTN {
                send_mouse_event(MouseEvent::Reset)?;
                return Ok(TransformStatus::Unchanged);
            } else if event.button == controller_state.SWITCH_MODE_BTN {
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

            if let Some(event_to_send) =
                match event.axis {
                    AxisName::PadX_SideL => {
                        Some(MouseEvent::LeftPad(PadStickEvent::MovedX(event.value)))
                    }
                    AxisName::PadY_SideL => {
                        Some(MouseEvent::LeftPad(PadStickEvent::MovedY(event.value)))
                    }
                    AxisName::PadX_SideR => {
                        Some(MouseEvent::RightPad(PadStickEvent::MovedX(event.value)))
                    }
                    AxisName::PadY_SideR => {
                        Some(MouseEvent::RightPad(PadStickEvent::MovedY(event.value)))
                    }
                    AxisName::StickX => {
                        Some(MouseEvent::Stick(PadStickEvent::MovedX(event.value)))
                    }
                    AxisName::StickY => {
                        Some(MouseEvent::Stick(PadStickEvent::MovedY(event.value)))
                    }
                    _ => {
                        None
                    }
                }
            {
                send_mouse_event(event_to_send)?;
                return Ok(TransformStatus::Handled);
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

lazy_static! {
    pub static ref TRIGGERS_RANGE_CONVERTER: RangeConverterBuilder<f32>  = RangeConverterBuilder::build(-1.0, 1.0, 0.0, 1.0);
}

pub fn transform_triggers(event: &mut TransformedEvent, configs: &Configs) -> TransformStatus {
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
                event.value = TRIGGERS_RANGE_CONVERTER.convert(event.value);

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
