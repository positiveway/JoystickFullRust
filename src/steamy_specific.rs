use crate::configs::MainConfigs;
use crate::match_event::{AxisName, ButtonName, EventTypeName, TransformStatus, TransformedEvent};
use crate::process_event::{process_event, ControllerState, ImplementationSpecificCfg};
use crate::steamy_debug::{buf_to_string, init_debug_files};
use crate::steamy_event::{SteamyButton, SteamyEvent, SteamyPadStickF32, SteamyTrigger};
use crate::steamy_state::SteamyState;
use log::{debug, error, warn};
use std::io::prelude::*;
use std::thread;
use std::thread::{JoinHandle, sleep};
use std::time::{Duration, Instant};
use color_eyre::eyre::{bail, Result};
use crate::utils::{check_thread_handle, ThreadHandleOption};
use crossbeam_channel::{bounded, Receiver, Sender};
// use kanal::{bounded, Receiver, Sender};

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

fn read_events(
    mut controller: steamy_base::Controller,
    configs: MainConfigs,
    steam_event_sender: SteamyEventSender,
) -> Result<()> {
    let steamy_read_interrupt_interval = configs.general.steamy_read_interrupt_interval;
    let input_raw_refresh_interval = configs.general.input_raw_refresh_interval;

    let mut state = SteamyState::default();

    //DEBUG
    let is_debug = configs.debugging_cfg.is_debug;
    let (mut subject_file, mut subject_endings_file, mut cmp_file) = init_debug_files()?;
    let mut msg_counter: u32 = 0;
    //DEBUG

    loop {
        let loop_start_time = Instant::now();

        msg_counter += 1;

        let (new_state, buffer) = controller.state(steamy_read_interrupt_interval)?;
        for event in state.update(new_state, buffer.clone(), &configs.layout_configs.axis_correction_cfg)? {
            debug!("{:?}", &event);

            if is_debug {
                match event {
                    SteamyEvent::PadStickF32(pad_stick_f32) => match pad_stick_f32 {
                        SteamyPadStickF32::LeftPadX(_)
                        | SteamyPadStickF32::LeftPadY(_)
                        | SteamyPadStickF32::StickX(_)
                        | SteamyPadStickF32::StickY(_) => {
                            let (content, ending) = buf_to_string(msg_counter, buffer.clone());
                            subject_file.write_all(content.as_bytes())?;

                            if ending != "" {
                                subject_endings_file.write_all(ending.as_bytes())?;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            steam_event_sender.send(event)?;
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
    writing_thread_handle: ThreadHandleOption,
    steamy_thread_handle: ThreadHandleOption,
) -> Result<()> {
    let impl_cfg = ImplementationSpecificCfg::new(0.0, 1.0);
    let input_buffer_refresh_interval = configs.general.input_buffer_refresh_interval;

    loop {
        let loop_start_time = Instant::now();

        if check_thread_handle(writing_thread_handle).is_err() {
            return Ok(())
        };

        if check_thread_handle(steamy_thread_handle).is_err() {
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
    writing_thread_handle: ThreadHandleOption,
) -> Result<()> {
    let mut manager = steamy_base::Manager::new()?;

    loop {
        let steamy_channel_size = configs.clone().general.steamy_channel_size;
        let configs_copy = configs.clone();

        match manager.open() {
            Ok(mut controller) => {
                println!("Gamepad connected");

                let (steam_event_sender, steam_event_receiver) = bounded(steamy_channel_size);

                let steamy_thread_handle = thread::spawn(move || {
                    read_events(controller, configs_copy, steam_event_sender).unwrap();
                });

                process_event_loop(
                    &mut controller_state,
                    configs.clone(),
                    steam_event_receiver,
                    writing_thread_handle,
                    Some(&steamy_thread_handle),
                )?;
            }
            Err(_) => {
                println!("Gamepad is not connected. Waiting...");
                sleep(Duration::from_millis(5000));
            }
        }
    }
}
