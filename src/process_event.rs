use crate::configs::{AxisCorrectionConfigs, LayoutConfigs, MainConfigs};
use crate::match_event::*;
use crate::math_ops::{apply_pad_stick_correction, RangeConverterBuilder};
use crate::process_event::ButtonEvent::{Pressed, Released};
use crate::process_event::PadStickEvent::{FingerLifted, FingerPut};
use color_eyre::eyre::{bail, Result};
use crate::utils::create_channel;
use log::debug;
use serde::{Deserialize, Serialize};
use strum_macros::Display;

#[cfg(not(feature = "use_kanal"))]
use crossbeam_channel::{Receiver, Sender};
#[cfg(feature = "use_kanal")]
use kanal::{Receiver, Sender};

#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum PadStickEvent {
    FingerPut,
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
    Pressed(ButtonName),
    Released(ButtonName),
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
    pub layout_configs: LayoutConfigs,
}

impl ControllerState {
    pub fn new(configs: MainConfigs) -> Self {
        let commands_channel_size = configs.general.commands_channel_size;

        let (mouse_sender, mouse_receiver) = create_channel(commands_channel_size);
        let (button_sender, button_receiver) = create_channel(commands_channel_size);

        let layout_configs = configs.layout_configs;
        Self {
            mouse_sender,
            mouse_receiver,
            button_sender,
            button_receiver,
            RESET_BTN: layout_configs.buttons_layout.reset_btn,
            SWITCH_MODE_BTN: layout_configs.buttons_layout.switch_mode_btn,
            layout_configs,
        }
    }

    pub fn release_all_hard(&self) -> Result<()> {
        self.button_sender.send(Released(self.RESET_BTN))?;
        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ImplementationSpecificCfg {
    triggers_range_converter: RangeConverterBuilder<f32>,
}

impl ImplementationSpecificCfg {
    pub fn new(trigger_input_min: f32, trigger_input_max: f32) -> Self {
        Self {
            triggers_range_converter: RangeConverterBuilder::build(
                trigger_input_min,
                trigger_input_max,
                0.0,
                1.0,
            ),
        }
    }
}

pub fn process_event(
    normalized_event: TransformStatus,
    controller_state: &mut ControllerState,
    impl_cfg: &ImplementationSpecificCfg,
) -> Result<()> {
    let mut event: TransformedEvent;
    match normalized_event {
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

    match transform_triggers(&mut event, &controller_state.layout_configs, impl_cfg) {
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

    match process_pad_stick(&mut event, controller_state)? {
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

pub fn process_buttons(
    event: &TransformedEvent,
    controller_state: &mut ControllerState,
) -> Result<TransformStatus> {
    match event.event_type {
        EventTypeName::ButtonPressed => {
            controller_state.button_sender.send(Pressed(event.button))?;
            Ok(TransformStatus::Handled)
        }
        EventTypeName::ButtonReleased => {
            controller_state
                .button_sender
                .send(Released(event.button))?;
            Ok(TransformStatus::Handled)
        }
        _ => Ok(TransformStatus::Unchanged),
    }
}

fn convert_to_pad_event(event_type: EventTypeName) -> Result<PadStickEvent> {
    let result = match event_type {
        EventTypeName::ButtonReleased => FingerLifted,
        EventTypeName::ButtonPressed => FingerPut,
        _ => bail!("Cannot happen")
    };
    debug!("{}", result);
    Ok(result)
}

fn apply_correction(
    event: &mut TransformedEvent,
    controller_state: &ControllerState,
) {
    let axis_correction_cfg = controller_state.layout_configs.axis_correction_cfg;

    let correction = match event.axis {
        AxisName::PadX_SideL => axis_correction_cfg.left_pad.x,
        AxisName::PadY_SideL => axis_correction_cfg.left_pad.y,
        AxisName::PadX_SideR => axis_correction_cfg.right_pad.x,
        AxisName::PadY_SideR => axis_correction_cfg.right_pad.y,
        AxisName::StickX => axis_correction_cfg.stick.x,
        AxisName::StickY => axis_correction_cfg.stick.y,
        _ => 0.0
    };

    match event.axis {
        AxisName::PadX_SideL
        | AxisName::PadY_SideL
        | AxisName::PadX_SideR
        | AxisName::PadY_SideR
        | AxisName::StickX
        | AxisName::StickY => {
            event.value = apply_pad_stick_correction(event.value, correction);
        }
        _ => {}
    }
}

pub fn process_pad_stick(
    event: &mut TransformedEvent,
    controller_state: &ControllerState,
) -> Result<TransformStatus> {
    let send_mouse_event = |mouse_event: MouseEvent| -> Result<()> {
        controller_state.mouse_sender.send(mouse_event)?;
        Ok(())
    };

    match event.button {
        // Important: Act only on Released event, not as Pressed
        ButtonName::PadAsTouch_SideR => {
            send_mouse_event(MouseEvent::RightPad(convert_to_pad_event(event.event_type)?))?;
            return Ok(TransformStatus::Handled);
        }
        ButtonName::PadAsTouch_SideL => {
            send_mouse_event(MouseEvent::LeftPad(convert_to_pad_event(event.event_type)?))?;
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
        #[cfg(not(feature = "use_steamy"))]{
            // Was needed for gilrs. Now causes various bugs
            // Discard 0.0 events for pads

            match event.axis {
                AxisName::PadX_SideL
                | AxisName::PadY_SideL
                | AxisName::PadX_SideR
                | AxisName::PadY_SideR => {
                    if event.value == 0.0 {
                        return Ok(TransformStatus::Discarded);
                    };
                }
                _ => {}
            }
        }

        apply_correction(event, controller_state);

        if let Some(event_to_send) = match event.axis {
            AxisName::PadX_SideL => Some(MouseEvent::LeftPad(PadStickEvent::MovedX(event.value))),
            AxisName::PadY_SideL => Some(MouseEvent::LeftPad(PadStickEvent::MovedY(event.value))),
            AxisName::PadX_SideR => Some(MouseEvent::RightPad(PadStickEvent::MovedX(event.value))),
            AxisName::PadY_SideR => Some(MouseEvent::RightPad(PadStickEvent::MovedY(event.value))),
            AxisName::StickX => Some(MouseEvent::Stick(PadStickEvent::MovedX(event.value))),
            AxisName::StickY => Some(MouseEvent::Stick(PadStickEvent::MovedY(event.value))),
            _ => None,
        } {
            send_mouse_event(event_to_send)?;
            return Ok(TransformStatus::Handled);
        };
    };

    Ok(TransformStatus::Unchanged)
}

pub fn transform_left_pad(event: &TransformedEvent) -> TransformStatus {
    match event.button {
        ButtonName::PadDown_SideL
        | ButtonName::PadRight_SideL
        | ButtonName::PadUp_SideL
        | ButtonName::PadLeft_SideL => TransformStatus::Transformed(TransformedEvent {
            event_type: event.event_type,
            axis: AxisName::None,
            value: event.value,
            button: ButtonName::PadAsBtn_SideL,
        }),
        _ => TransformStatus::Unchanged,
    }
}

pub fn transform_triggers(
    event: &mut TransformedEvent,
    layout_configs: &LayoutConfigs,
    impl_cfg: &ImplementationSpecificCfg,
) -> TransformStatus {
    match event.button {
        ButtonName::LowerTriggerAsBtn_SideL | ButtonName::LowerTriggerAsBtn_SideR => {
            return TransformStatus::Discarded;
        }
        _ => {}
    };

    match event.axis {
        AxisName::LowerTrigger_SideL | AxisName::LowerTrigger_SideR => {
            let button = match event.axis {
                AxisName::LowerTrigger_SideL => ButtonName::LowerTriggerAsBtn_SideL,
                AxisName::LowerTrigger_SideR => ButtonName::LowerTriggerAsBtn_SideR,
                _ => ButtonName::None,
            };
            return TransformStatus::Transformed({
                event.value = impl_cfg.triggers_range_converter.convert(event.value);

                if event.value > layout_configs.general.triggers_threshold {
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
