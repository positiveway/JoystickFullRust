use crate::configs::MainConfigs;
use crate::match_event::{AxisName, ButtonName, EventTypeName, TransformStatus, TransformedEvent};
use crate::process_event::{process_event, ControllerState, ImplementationSpecificCfg};
use crate::steamy_event::{SteamyButton, SteamyEvent, SteamyPadStickF32, SteamyTrigger};
use crate::steamy_state::SteamyState;
use log::{debug, error, warn};
use std::io::prelude::*;
use std::thread;
use std::thread::{sleep};
use std::time::{Duration, Instant};
use color_eyre::eyre::{bail, Result};
use crate::utils::{create_channel, TerminationStatus};

#[cfg(not(feature = "use_kanal"))]
use crossbeam_channel::{Receiver, Sender};
#[cfg(feature = "use_kanal")]
use kanal::{Receiver, Sender};


pub type SteamyEventSender = Sender<SteamyEvent>;
pub type SteamyEventReceiver = Receiver<SteamyEvent>;

pub fn match_button(button: &SteamyButton) -> Result<ButtonName> {
    Ok(match button {
        SteamyButton::A => ButtonName::BtnDown_SideR,
        SteamyButton::B => ButtonName::BtnRight_SideR,
        SteamyButton::X => ButtonName::BtnLeft_SideR,
        SteamyButton::Y => ButtonName::BtnUp_SideR,
        SteamyButton::Down => ButtonName::PadDown_SideL,
        SteamyButton::Left => ButtonName::PadLeft_SideL,
        SteamyButton::Right => ButtonName::PadRight_SideL,
        SteamyButton::Up => ButtonName::PadUp_SideL,
        SteamyButton::LeftPadPressed => ButtonName::PadAsBtn_SideL,
        SteamyButton::LeftPadTouch => ButtonName::PadAsTouch_SideL,
        SteamyButton::StickTouch => {
            // println!("StickTouch happened");
            warn!("StickTouch happened");
            ButtonName::None
        },
        SteamyButton::StickPressed => ButtonName::StickAsBtn,
        SteamyButton::RightPadPressed => ButtonName::PadAsBtn_SideR,
        SteamyButton::RightPadTouch => ButtonName::PadAsTouch_SideR,
        SteamyButton::Back => ButtonName::ExtraBtn_SideL,
        SteamyButton::Home => ButtonName::ExtraBtnCentral,
        SteamyButton::Forward => ButtonName::ExtraBtn_SideR,
        SteamyButton::BumperLeft => ButtonName::UpperTrigger_SideL,
        SteamyButton::BumperRight => ButtonName::UpperTrigger_SideR,
        SteamyButton::GripLeft => ButtonName::Wing_SideL,
        SteamyButton::GripRight => ButtonName::Wing_SideR,
        SteamyButton::TriggerLeft => ButtonName::LowerTriggerAsBtn_SideL,
        SteamyButton::TriggerRight => ButtonName::LowerTriggerAsBtn_SideR,
    })
}

pub fn normalize_event(
    event: &SteamyEvent,
    RESET_BTN: ButtonName,
) -> Result<TransformStatus> {
    Ok(match event {
        SteamyEvent::Button(button, pressed) => {
            let button = match_button(button)?;
            if button == ButtonName::None {
                TransformStatus::Discarded
            } else {
                TransformStatus::Transformed(TransformedEvent {
                    event_type: match pressed {
                        true => EventTypeName::ButtonPressed,
                        false => EventTypeName::ButtonReleased,
                    },
                    axis: AxisName::None,
                    button,
                    value: 0.0,
                })
            }
        }
        SteamyEvent::Trigger(trigger) => TransformStatus::Transformed(match trigger {
            SteamyTrigger::Left(value) => TransformedEvent {
                event_type: EventTypeName::AxisChanged,
                axis: AxisName::LowerTrigger_SideL,
                value: *value,
                button: ButtonName::None,
            },
            SteamyTrigger::Right(value) => TransformedEvent {
                event_type: EventTypeName::AxisChanged,
                axis: AxisName::LowerTrigger_SideR,
                value: *value,
                button: ButtonName::None,
            },
        }),
        SteamyEvent::PadStickF32(pad_stick_f32) => {
            TransformStatus::Transformed(match pad_stick_f32 {
                SteamyPadStickF32::LeftPadX(value) => TransformedEvent {
                    event_type: EventTypeName::AxisChanged,
                    axis: AxisName::PadX_SideL,
                    value: *value,
                    button: ButtonName::None,
                },
                SteamyPadStickF32::LeftPadY(value) => TransformedEvent {
                    event_type: EventTypeName::AxisChanged,
                    axis: AxisName::PadY_SideL,
                    value: *value,
                    button: ButtonName::None,
                },
                SteamyPadStickF32::RightPadX(value) => TransformedEvent {
                    event_type: EventTypeName::AxisChanged,
                    axis: AxisName::PadX_SideR,
                    value: *value,
                    button: ButtonName::None,
                },
                SteamyPadStickF32::RightPadY(value) => TransformedEvent {
                    event_type: EventTypeName::AxisChanged,
                    axis: AxisName::PadY_SideR,
                    value: *value,
                    button: ButtonName::None,
                },
                SteamyPadStickF32::StickX(value) => TransformedEvent {
                    event_type: EventTypeName::AxisChanged,
                    axis: AxisName::StickX,
                    value: *value,
                    button: ButtonName::None,
                },
                SteamyPadStickF32::StickY(value) => TransformedEvent {
                    event_type: EventTypeName::AxisChanged,
                    axis: AxisName::StickY,
                    value: *value,
                    button: ButtonName::None,
                },
            })
        }
        SteamyEvent::Disconnected => TransformStatus::Transformed(TransformedEvent {
            event_type: EventTypeName::ButtonReleased,
            axis: AxisName::None,
            button: RESET_BTN,
            value: 0.0,
        }),
        _ => TransformStatus::Discarded,
    })
}

fn read_events_loop(
    mut controller: steamy_base::Controller,
    configs: MainConfigs,
    steam_event_sender: SteamyEventSender,
    termination_status: TerminationStatus,
) -> Result<()> {
    let steamy_read_interrupt_interval = configs.general.steamy_read_interrupt_interval;
    let input_raw_refresh_interval = configs.general.input_raw_refresh_interval;
    let steamy_use_left_pad = configs.layout_configs.general.steamy_use_left_pad;

    let mut state = SteamyState::default();

    // DEBUG
    use crate::steamy_debug::{buf_to_string, init_debug_files};
    let (mut subject_file, mut subject_endings_file, mut cmp_file) = init_debug_files(configs.is_left_pad_bytes_dump)?;
    let mut msg_counter: u32 = 0;
    // DEBUG

    let mut left_pad_active = false;

    loop {
        let loop_start_time = Instant::now();

        if termination_status.check() {
            return Ok(())
        };

        #[cfg(not(feature = "debug_mode"))]{
            let (new_state, is_left_pad) = controller.state(steamy_read_interrupt_interval)?;
            for event in state.update(new_state, steamy_use_left_pad)? {
                // for event in state.update(new_state, is_left_pad)? {
                // for event in state.update(new_state, left_pad_active)? {
                match event {
                    SteamyEvent::Button(button, pressed) => {
                        match button {
                            SteamyButton::LeftPadTouch => {
                                left_pad_active = pressed;
                            },
                            SteamyButton::StickTouch | SteamyButton::StickPressed => {
                                println!("Stick touch: {}", pressed)
                            },
                            _ => {}
                        }
                    }
                    _ => {}
                }
                steam_event_sender.send(event)?;
            }
        }

        #[cfg(feature = "debug_mode")] {
            msg_counter += 1;

            let (new_state, buffer) = controller.state(steamy_read_interrupt_interval)?;
            for event in state.update(new_state, false, &configs.layout_configs.axis_correction_cfg)? {
                debug!("{:?}", &event);

                match event {
                    SteamyEvent::PadStickF32(pad_stick_f32) => match pad_stick_f32 {
                        SteamyPadStickF32::LeftPadX(_)
                        | SteamyPadStickF32::LeftPadY(_)
                        | SteamyPadStickF32::StickX(_)
                        | SteamyPadStickF32::StickY(_) => {
                            let (content, ending) = buf_to_string(msg_counter, &buffer);
                            subject_file.write_all(content.as_bytes())?;

                            if ending != "" {
                                subject_endings_file.write_all(ending.as_bytes())?;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }

                steam_event_sender.send(event)?;
            }
        }

        let loop_iteration_runtime = loop_start_time.elapsed();

        if let Some(remaining) = input_raw_refresh_interval.checked_sub(loop_iteration_runtime) {
            sleep(remaining);
        }
    }
}

fn process_event_loop(
    controller_state: &mut ControllerState,
    configs: MainConfigs,
    steam_event_receiver: SteamyEventReceiver,
    termination_status: TerminationStatus,
) -> Result<()> {
    let impl_cfg = ImplementationSpecificCfg::new(0.0, 1.0);
    let input_buffer_refresh_interval = configs.general.input_buffer_refresh_interval;

    loop {
        let loop_start_time = Instant::now();

        if termination_status.check() {
            return Ok(())
        };

        for event in steam_event_receiver.try_iter() {
            // while let Some(event) = steam_event_receiver.try_recv()? {
            let is_disconnected = event == SteamyEvent::Disconnected;

            let event = normalize_event(&event, controller_state.RESET_BTN)?;
            process_event(event, controller_state, &impl_cfg)?;

            if is_disconnected {
                controller_state.release_all_hard()?;
                println!("Gamepad disconnected");
                return Ok(());
            }
        }

        let loop_iteration_runtime = loop_start_time.elapsed();

        if let Some(remaining) = input_buffer_refresh_interval.checked_sub(loop_iteration_runtime) {
            sleep(remaining);
        }
    }
}

pub fn run_steamy_loop(
    mut controller_state: ControllerState,
    configs: MainConfigs,
    termination_status: TerminationStatus,
) -> Result<()> {
    let steamy_channel_size = configs.clone().general.steamy_channel_size;
    let (steam_event_sender, steam_event_receiver) = create_channel(steamy_channel_size);

    let mut manager = steamy_base::Manager::new()?;

    loop {
        let configs_copy = configs.clone();
        let configs_copy2 = configs.clone();
        let steam_event_sender_copy = steam_event_sender.clone();
        let steam_event_receiver_copy = steam_event_receiver.clone();
        let termination_status_copy = termination_status.clone();
        let termination_status_copy2 = termination_status.clone();
        let mut controller_state_copy = controller_state.clone();

        match manager.open() {
            Ok(controller) => {
                println!("Gamepad connected");

                termination_status.spawn_with_check(
                    move || -> Result<()>{
                        read_events_loop(
                            controller,
                            configs_copy,
                            steam_event_sender_copy,
                            termination_status_copy,
                        )
                    }
                );

                termination_status.run_with_check(
                    move || -> Result<()>{
                        process_event_loop(
                            &mut controller_state_copy,
                            configs_copy2,
                            steam_event_receiver_copy,
                            termination_status_copy2,
                        )
                    }
                );
            }
            Err(_) => {
                println!("Gamepad is not connected. Waiting...");
                sleep(Duration::from_millis(5000));
            }
        }
    }
}
