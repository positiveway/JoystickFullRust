use gilrs::EventType;
use color_eyre::eyre::{bail, OptionExt, Result};
use serde::{Deserialize, Serialize};
use crate::match_event::*;
use crate::configs::{LayoutConfigs, MainConfigs};
use crossbeam_channel::{Sender, Receiver, bounded};
use lazy_static::lazy_static;
use mouse_keyboard_input::Button;
use strum_macros::{Display};
use crate::buttons_state::ButtonsState;
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

#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum ButtonEvent {
    Pressed(Button),
    Released(Button),
}

pub type MouseSender = Sender<MouseEvent>;
pub type MouseReceiver = Receiver<MouseEvent>;

pub type ButtonSender = Sender<ButtonEvent>;
pub type ButtonReceiver = Receiver<ButtonEvent>;

#[derive(Clone, Debug)]
pub struct ControllerState {
    pub mouse_sender: MouseSender,
    pub mouse_receiver: MouseReceiver,
    pub button_sender: ButtonSender,
    pub button_receiver: ButtonReceiver,
    //
    pub RESET_BTN: ButtonName,
    pub SWITCH_MODE_BTN: ButtonName,
    //
    pub buttons_state: ButtonsState,
    //
    pub layout_configs: LayoutConfigs,
}

impl ControllerState {
    pub fn new(configs: MainConfigs) -> Self {
        let (mouse_sender, mouse_receiver) = bounded(configs.channel_size);
        let (button_sender, button_receiver) = bounded(configs.channel_size);
        let layout_configs = configs.layout_configs;
        Self {
            mouse_sender,
            mouse_receiver,
            button_sender: button_sender.clone(),
            button_receiver,
            RESET_BTN: layout_configs.buttons_layout.reset_btn,
            SWITCH_MODE_BTN: layout_configs.buttons_layout.switch_mode_btn,
            buttons_state: ButtonsState::new(layout_configs.buttons_layout.clone(), button_sender),
            layout_configs,
        }
    }
}


pub enum TransformStatus {
    Discarded,
    Unchanged,
    Transformed(TransformedEvent),
    Handled,
}

pub fn process_event(orig_event: &EventType, controller_state: &mut ControllerState) -> Result<()> {
    let mut event: TransformedEvent;
    match match_event(orig_event, controller_state.RESET_BTN)? {
        TransformStatus::Discarded => {
            return Ok(());
        }
        TransformStatus::Transformed(transformed_event) => {
            event = transformed_event;
        }
        TransformStatus::Unchanged | TransformStatus::Handled => {
            bail!("Forbidden status")
        }
    };


    match transform_triggers(&mut event, &controller_state.layout_configs) {
        TransformStatus::Discarded | TransformStatus::Handled => {
            return Ok(());
        }
        TransformStatus::Transformed(transformed_event) => {
            event = transformed_event;
        }
        TransformStatus::Unchanged => {}
    };

    match transform_left_pad(&event, controller_state.layout_configs.gaming_mode) {
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

    match process_buttons(&event, controller_state)? {
        TransformStatus::Discarded | TransformStatus::Handled => {
            return Ok(());
        }
        TransformStatus::Transformed(transformed_event) => {
            event = transformed_event;
        }
        TransformStatus::Unchanged => {}
    }

    Ok(())
}

pub fn process_buttons(event: &TransformedEvent, controller_state: &mut ControllerState) -> Result<TransformStatus> {
    match event.event_type {
        EventTypeName::ButtonReleased => {
            controller_state.buttons_state.release(event.button)?;
            Ok(TransformStatus::Handled)
        }
        EventTypeName::ButtonPressed => {
            controller_state.buttons_state.press(event.button)?;
            Ok(TransformStatus::Handled)
        }
        _ => {
            Ok(TransformStatus::Unchanged)
        }
    }
}


pub fn process_pad_stick(event: &TransformedEvent, controller_state: &ControllerState) -> Result<TransformStatus> {
    let send_mouse_event = |mouse_event: MouseEvent| -> Result<()> {
        exec_or_eyre!(controller_state.mouse_sender.send(mouse_event))
    };

    match event.button {
        // Important: Act only on Released event, not as Pressed
        ButtonName::PadAsTouch_SideR => {
            if event.event_type == EventTypeName::ButtonReleased {
                send_mouse_event(MouseEvent::RightPad(FingerLifted))?;
            }
            return Ok(TransformStatus::Handled);
        }
        ButtonName::PadAsTouch_SideL => {
            if event.event_type == EventTypeName::ButtonReleased {
                send_mouse_event(MouseEvent::LeftPad(FingerLifted))?;
            }
            return Ok(TransformStatus::Handled);
        }
        _ => {
            if event.button == controller_state.RESET_BTN {
                if event.event_type == EventTypeName::ButtonReleased {
                    send_mouse_event(MouseEvent::Reset)?;
                }
                return Ok(TransformStatus::Unchanged);
            } else if event.button == controller_state.SWITCH_MODE_BTN {
                if event.event_type == EventTypeName::ButtonReleased {
                    send_mouse_event(MouseEvent::ModeSwitched)?;
                }
                return Ok(TransformStatus::Handled);
            };
        }
    };

    if event.event_type == EventTypeName::AxisChanged {
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
    };

    Ok(TransformStatus::Unchanged)
}


pub fn transform_left_pad(event: &TransformedEvent, gaming_mode: bool) -> TransformStatus {
    match event.event_type {
        EventTypeName::ButtonReleased | EventTypeName::ButtonPressed => {
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
        EventTypeName::AxisChanged => {
            if gaming_mode {}
            TransformStatus::Unchanged
        }
    }
}

lazy_static! {
    pub static ref TRIGGERS_RANGE_CONVERTER: RangeConverterBuilder<f32>  = RangeConverterBuilder::build(-1.0, 1.0, 0.0, 1.0);
}

pub fn transform_triggers(event: &mut TransformedEvent, layout_configs: &LayoutConfigs) -> TransformStatus {
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

                if event.value > layout_configs.triggers_threshold {
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
