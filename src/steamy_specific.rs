use crate::configs::MainConfigs;
use crate::match_event::{AxisName, ButtonName, EventTypeName, TransformStatus, TransformedEvent};
use crate::process_event::{process_event, ImplementationSpecificCfg, SharedInfo};
use crate::steamy_event::{SteamyButton, SteamyEvent, SteamyPadStickF32, SteamyTrigger};
use crate::steamy_state::SteamyState;
use crate::utils::{create_channel, TerminationStatus};
use color_eyre::eyre::{bail, Result};
use log::{debug, error, warn};
use std::fs::File;
use std::io::prelude::*;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

#[cfg(not(feature = "use_kanal"))]
use crossbeam_channel::{Receiver, Sender};
#[cfg(feature = "use_kanal")]
use kanal::{Receiver, Sender};
use steamy_base::Manager;

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
        }
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

pub fn normalize_event(event: &SteamyEvent, RESET_BTN: ButtonName) -> Result<TransformStatus> {
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

#[inline(always)]
fn read_events(
    controller: &mut steamy_base::Controller,
    configs: &MainConfigs,
    steam_event_sender: &SteamyEventSender,
    state: &mut SteamyState,
    left_pad_active: &mut bool,
    subject_file: &mut File,
    subject_endings_file: &mut File,
    cmp_file: &mut File,
    msg_counter: &mut u32,
) -> Result<Vec<SteamyEvent>> {
    let steamy_read_interrupt_interval = configs.general.steamy_read_interrupt_interval;
    let steamy_use_left_pad = configs.layout_configs.general.steamy_use_left_pad;

    let mut received_events = vec![];

    #[cfg(not(feature = "debug_mode"))]
    {
        let (new_state, is_left_pad) = controller.state(steamy_read_interrupt_interval)?;
        for event in state.update(new_state, steamy_use_left_pad)? {
            // for event in state.update(new_state, is_left_pad)? {
            // for event in state.update(new_state, *left_pad_active)? {
            match event {
                SteamyEvent::Button(button, pressed) => match button {
                    SteamyButton::LeftPadTouch => {
                        *left_pad_active = pressed;
                    }
                    SteamyButton::StickTouch | SteamyButton::StickPressed => {
                        println!("Stick touch: {}", pressed)
                    }
                    _ => {}
                },
                _ => {}
            }
            // received_events.push(event);
            steam_event_sender.send(event)?;
        }
    }

    #[cfg(feature = "debug_mode")]
    {
        use crate::steamy_debug::buf_to_string;

        *msg_counter += 1;

        let (new_state, buffer) = controller.state(steamy_read_interrupt_interval)?;
        for event in state.update(new_state, false)? {
            debug!("{:?}", &event);

            match event {
                SteamyEvent::PadStickF32(pad_stick_f32) => match pad_stick_f32 {
                    SteamyPadStickF32::LeftPadX(_)
                    | SteamyPadStickF32::LeftPadY(_)
                    | SteamyPadStickF32::StickX(_)
                    | SteamyPadStickF32::StickY(_) => {
                        let (content, ending) = buf_to_string(*msg_counter, &buffer);
                        subject_file.write_all(content.as_bytes())?;

                        if ending != "" {
                            subject_endings_file.write_all(ending.as_bytes())?;
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            // received_events.push(event);
            steam_event_sender.send(event)?;
        }
    }

    Ok(received_events)
}

#[cfg(feature = "steamy_use_threads")]
fn read_events_loop(
    controller: &mut steamy_base::Controller,
    configs: &MainConfigs,
    steam_event_sender: &SteamyEventSender,
    termination_status: &TerminationStatus,
) -> Result<()> {
    // let steamy_read_interrupt_interval = configs.general.steamy_read_interrupt_interval;
    let input_raw_refresh_interval = configs.general.input_raw_refresh_interval;
    // let steamy_use_left_pad = configs.layout_configs.general.steamy_use_left_pad;

    let mut state = SteamyState::default();

    // DEBUG
    use crate::steamy_debug::init_debug_files;
    let (mut subject_file, mut subject_endings_file, mut cmp_file) =
        init_debug_files(configs.is_left_pad_bytes_dump)?;
    let mut msg_counter: u32 = 0;
    // DEBUG

    let mut left_pad_active = false;

    loop {
        let loop_start_time = Instant::now();

        if termination_status.check() {
            return Ok(());
        };

        let received_events = read_events(
            controller,
            configs,
            steam_event_sender,
            &mut state,
            &mut left_pad_active,
            &mut subject_file,
            &mut subject_endings_file,
            &mut cmp_file,
            &mut msg_counter,
        )?;

        let loop_iteration_runtime = loop_start_time.elapsed();

        if let Some(remaining) = input_raw_refresh_interval.checked_sub(loop_iteration_runtime) {
            sleep(remaining);
        }
    }
}

#[inline(always)]
fn process_events(
    shared_info: &SharedInfo,
    impl_cfg: &ImplementationSpecificCfg,
    steam_event_receiver: &SteamyEventReceiver,
) -> Result<bool> {
    for event in steam_event_receiver.try_iter() {
        // while let Some(event) = steam_event_receiver.try_recv()? {
        let is_disconnected = event == SteamyEvent::Disconnected;

        let event = normalize_event(&event, shared_info.RESET_BTN)?;
        process_event(event, shared_info, &impl_cfg)?;

        if is_disconnected {
            shared_info.release_all_hard()?;
            println!("Gamepad disconnected");
            return Ok(true);
        }
    }

    Ok(false)
}

#[cfg(feature = "steamy_use_threads")]
fn process_event_loop(
    shared_info: &SharedInfo,
    configs: &MainConfigs,
    steam_event_receiver: &SteamyEventReceiver,
    termination_status: &TerminationStatus,
) -> Result<()> {
    let impl_cfg = ImplementationSpecificCfg::new(0.0, 1.0);
    let input_buffer_refresh_interval = configs.general.input_buffer_refresh_interval;

    loop {
        let loop_start_time = Instant::now();

        if termination_status.check() {
            return Ok(());
        };

        let exit_now = process_events(shared_info, &impl_cfg, steam_event_receiver)?;
        if exit_now {
            return Ok(());
        }

        let loop_iteration_runtime = loop_start_time.elapsed();

        if let Some(remaining) = input_buffer_refresh_interval.checked_sub(loop_iteration_runtime) {
            sleep(remaining);
        }
    }
}

#[cfg(not(feature = "steamy_use_threads"))]
fn process_event_loop(
    controller: &mut steamy_base::Controller,
    shared_info: &SharedInfo,
    configs: &MainConfigs,
    steam_event_sender: &SteamyEventSender,
    steam_event_receiver: &SteamyEventReceiver,
    termination_status: &TerminationStatus,
) -> Result<()> {
    let impl_cfg = ImplementationSpecificCfg::new(0.0, 1.0);

    let input_buffer_refresh_interval = configs.general.input_buffer_refresh_interval;
    let input_raw_refresh_interval = configs.general.input_raw_refresh_interval;

    let mut state = SteamyState::default();

    // DEBUG
    use crate::steamy_debug::{buf_to_string, init_debug_files};
    let (mut subject_file, mut subject_endings_file, mut cmp_file) =
        init_debug_files(configs.is_left_pad_bytes_dump)?;
    let mut msg_counter: u32 = 0;
    // DEBUG

    let mut left_pad_active = false;

    let mut process_loop_start_time = Instant::now();
    let mut events_buffer: Vec<SteamyEvent> = vec![];

    loop {
        let read_loop_start_time = Instant::now();

        if termination_status.check() {
            return Ok(());
        };

        let received_events = read_events(
            controller,
            configs,
            steam_event_sender,
            &mut state,
            &mut left_pad_active,
            &mut subject_file,
            &mut subject_endings_file,
            &mut cmp_file,
            &mut msg_counter,
        )?;
        // events_buffer.extend(received_events);

        let process_loop_iteration_runtime = process_loop_start_time.elapsed();

        if input_buffer_refresh_interval.checked_sub(process_loop_iteration_runtime) == None {
            process_loop_start_time = Instant::now();

            let exit_now = process_events(shared_info, &impl_cfg, steam_event_receiver)?;
            if exit_now {
                return Ok(());
            }
        }

        let read_loop_iteration_runtime = read_loop_start_time.elapsed();

        if let Some(remaining) = input_raw_refresh_interval.checked_sub(read_loop_iteration_runtime)
        {
            sleep(remaining);
        }
    }
}

fn wait_for_connection() -> Result<()> {
    let mut manager = steamy_base::Manager::new()?;

    loop {
        match manager.open() {
            Ok(_) => {
                println!("Gamepad connected");
                return Ok(());
            }
            Err(_) => {
                println!("Gamepad is not connected. Waiting...");
                sleep(Duration::from_millis(5000));
            }
        }
    }
}

pub fn run_steamy_loop(
    shared_info: &SharedInfo,
    configs: &MainConfigs,
    termination_status: &TerminationStatus,
) -> Result<()> {
    wait_for_connection()?;

    let steamy_channel_size = configs.clone().general.steamy_channel_size;
    let (steam_event_sender, steam_event_receiver) = create_channel(steamy_channel_size);

    let mut manager: Manager = Manager::new()?;

    let configs_copy = configs.clone();
    let termination_status_copy = termination_status.clone();

    match manager.open() {
        Ok(mut controller) => {
            #[cfg(not(feature = "steamy_use_threads"))]
            {
                termination_status.check_result(process_event_loop(
                    &mut controller,
                    &shared_info,
                    &configs,
                    &steam_event_sender,
                    &steam_event_receiver,
                    &termination_status,
                ));
            }

            #[cfg(feature = "steamy_use_threads")]
            {
                thread::spawn(move || {
                    termination_status_copy.check_result(read_events_loop(
                        &mut controller,
                        &configs_copy,
                        &steam_event_sender,
                        &termination_status_copy,
                    ));
                });

                termination_status.check_result(process_event_loop(
                    &shared_info,
                    &configs,
                    &steam_event_receiver,
                    &termination_status,
                ));
            }
        }
        Err(_) => {
            bail!("Cannot happen")
        }
    };

    Ok(())
}
