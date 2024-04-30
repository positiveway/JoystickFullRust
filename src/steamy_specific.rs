use crate::configs::MainConfigs;
use crate::exec_or_eyre;
use crate::match_event::{AxisName, ButtonName, EventTypeName, TransformStatus, TransformedEvent};
use crate::process_event::{process_event, ControllerState};
use log::debug;
use crate::steamy_event::{SteamyButton, SteamyEvent, SteamyPadStickF32, SteamyTrigger};
use std::fs::{read_dir, remove_file, File, OpenOptions};
use std::io::prelude::*;
use std::thread::sleep;
use std::time::Duration;
use crate::steamy_state::SteamyState;

const IS_PAD: bool = false;

const BUF_SIZE: usize = 60;
const BASE_PATH: &str = "/home/user/Documents/bytes";

pub fn match_button(button: &SteamyButton) -> color_eyre::Result<ButtonName> {
    Ok(match button {
        SteamyButton::A => ButtonName::BtnDown_SideR,
        SteamyButton::B => ButtonName::BtnRight_SideR,
        SteamyButton::X => ButtonName::BtnLeft_SideR,
        SteamyButton::Y => ButtonName::BtnUp_SideR,
        SteamyButton::Down => ButtonName::PadDown_SideL,
        SteamyButton::Left => ButtonName::PadLeft_SideL,
        SteamyButton::Right => ButtonName::PadRight_SideL,
        SteamyButton::Up => ButtonName::PadUp_SideL,
        SteamyButton::Pad => ButtonName::None, //unreliable: triggered for both stick and left pad
        SteamyButton::PadTouch => ButtonName::PadAsTouch_SideL,
        SteamyButton::Stick => ButtonName::StickAsBtn,
        SteamyButton::StickTouch => ButtonName::None,
        SteamyButton::Track => ButtonName::None,
        SteamyButton::TrackTouch => ButtonName::PadAsTouch_SideR,
        SteamyButton::Back => ButtonName::ExtraBtn_SideL,
        SteamyButton::Home => ButtonName::ExtraBtnCentral,
        SteamyButton::Forward => ButtonName::ExtraBtn_SideR,
        SteamyButton::BumperLeft => ButtonName::UpperTrigger_SideL,
        SteamyButton::BumperRight => ButtonName::UpperTrigger_SideR,
        SteamyButton::GripLeft => ButtonName::Wing_SideL,
        SteamyButton::GripRight => ButtonName::Wing_SideR,
        SteamyButton::TriggerLeft => ButtonName::None,
        SteamyButton::TriggerRight => ButtonName::None,
    })
}

pub fn normalize_event(
    event: &SteamyEvent,
    buffer: Vec<u8>,
    RESET_BTN: ButtonName,
) -> color_eyre::Result<TransformStatus> {
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
        SteamyEvent::PadStickF32(pad_stick_f32) => TransformStatus::Transformed(match pad_stick_f32 {
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
        }),
        SteamyEvent::Disconnected => TransformStatus::Transformed(TransformedEvent {
            event_type: EventTypeName::ButtonReleased,
            axis: AxisName::None,
            button: RESET_BTN,
            value: 0.0,
        }),
        _ => TransformStatus::Discarded,
    })
}

fn align_num(val: String, padding: usize) -> String {
    let mut res = String::from("");
    if val.len() >= padding {
        val
    } else {
        for _ in 0..(padding - val.len()) {
            res.push(' ')
        }
        res.push_str(val.as_str());
        res
    }
}

fn buf_to_string_raw(msg_counter: u32, buf: Vec<u8>) -> String {
    let mut res = format!("[{}] ", align_num(format!("{}", msg_counter), 3));
    for (ind, num) in buf.iter().enumerate() {
        let num = align_num(format!("{:08b}", num), 8);
        res.push_str(format!("{}|", num).as_str());
    }
    res.push('\n');
    res
}

fn buf_to_string(msg_counter: u32, buf: Vec<u8>) -> (String, String) {
    let res = buf_to_string_raw(msg_counter, buf.clone());

    if &buf[12..=15] == [0, 0, 0, 0] {
        (format!("{}{}", res, get_header()), res)
    } else {
        (res, String::from(""))
    }
}

fn get_separator() -> String {
    let mut res = String::from("");
    for _ in 0..(BUF_SIZE * 9 + 5) {
        res.push('-');
    }
    res.push('\n');
    res
}

fn get_header() -> String {
    let mut content = get_separator();
    content.push_str("[   ] ");
    for i in 0..BUF_SIZE {
        content.push_str(format!("{}|", align_num(format!("{}", i), 8)).as_str());
    }
    content.push('\n');
    content.push_str(get_separator().as_str());
    content
}

fn clean_dir_fn(dir_path: String) -> color_eyre::Result<()> {
    for entry in read_dir(dir_path)? {
        remove_file(entry?.path())?;
    }
    Ok(())
}

fn create_file(subject: &str, endings: bool) -> color_eyre::Result<File> {
    let dir_path = format!("{}/{}", BASE_PATH, subject);

    if !endings {
        clean_dir_fn(dir_path.clone())?;
    }

    let subject = if endings {
        format!("{}_endings", subject)
    } else {
        subject.to_string()
    };

    let mut file = OpenOptions::new()
        // .append(true)
        // .create(true)
        .write(true)
        .create_new(true)
        .open(format!("{}/{}.txt", dir_path, subject))?;
    file.write_all(get_header().as_bytes())?;

    Ok(file)
}

fn read_events(
    mut controller: steamy_base::Controller,
    controller_state: &mut ControllerState,
    configs: MainConfigs,
) -> color_eyre::Result<()> {
    //DEBUG
    let subject = if IS_PAD { "pad" } else { "stick" };

    let mut subject_file = create_file(subject, false)?;
    let mut subject_endings_file = create_file(subject, true)?;

    let mut cmp_file = create_file("cmp", false)?;
    cmp_file.write_all("\n".as_bytes())?;
    cmp_file.write_all(get_separator().as_bytes())?;
    //DEBUG

    let mut state = SteamyState::default();
    let mut msg_counter: u32 = 0;

    loop {
        msg_counter += 1;

        let (new_state, buffer) = controller.state(Duration::from_secs(0))?;
        for event in state.update(new_state, buffer.clone()) {
            debug!("{:?}", &event);
            let is_disconnected = event == SteamyEvent::Disconnected;

            match event {
                SteamyEvent::PadStickF32(pad_stick_f32) => match pad_stick_f32 {
                    SteamyPadStickF32::LeftPadX(_)
                    | SteamyPadStickF32::LeftPadY(_)
                    | SteamyPadStickF32::StickX(_)
                    | SteamyPadStickF32::StickY(_) => {
                        if configs.debug {
                            let (content, ending) = buf_to_string(msg_counter, buffer.clone());
                            subject_file.write_all(content.as_bytes())?;

                            if ending != "" {
                                subject_endings_file.write_all(ending.as_bytes())?;
                            }
                        }
                    }
                    _ => {}
                },
                _ => {}
            }

            let event = normalize_event(&event, buffer.clone(), controller_state.RESET_BTN)?;
            process_event(event, controller_state)?;

            if is_disconnected {
                println!("Gamepad disconnected");
                return Ok(());
            }
        }

        sleep(Duration::from_millis(4)); //4 = USB min latency
    }
}

pub fn run_steamy_loop(
    mut controller_state: ControllerState,
    configs: MainConfigs,
) -> color_eyre::Result<()> {
    let mut manager = exec_or_eyre!(steamy_base::Manager::new())?;

    loop {
        match manager.open() {
            Ok(mut controller) => {
                read_events(controller, &mut controller_state, configs.clone())?;
            }
            Err(_) => {
                println!("Gamepad is not connected. Waiting...");
                sleep(Duration::from_millis(5000));
            }
        }
    }
}