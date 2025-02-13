use crate::configs::MainConfigs;
use crate::exec_or_eyre;
use crate::match_event::{AxisName, ButtonName, EventTypeName, TransformStatus, TransformedEvent};
use crate::process_event::{process_event, ImplementationSpecificCfg, SharedInfo};
use crate::utils::TerminationStatus;
use color_eyre::eyre::{bail, OptionExt, Result};
use gilrs::ev::Code;
use gilrs::EventType::Disconnected;
use gilrs::{Axis, Button, Event, EventType, EventType::*, Filter, Gamepad, Gilrs};
use log::debug;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::thread::sleep;
use std::time::Duration;

const VENDOR_ID: u16 = 0x28de;
const PRODUCT_ID: [u16; 2] = [0x1102, 0x1142];
const ENDPOINT: [u8; 2] = [3, 2];
const INDEX: [u16; 2] = [2, 1];

struct UsbHolder {
    device: rusb::Device<rusb::GlobalContext>,
    handle: rusb::DeviceHandle<rusb::GlobalContext>,
    address: u8,
}

fn detach_driver(
    mut device: rusb::Device<rusb::GlobalContext>,
    mut handle: rusb::DeviceHandle<rusb::GlobalContext>,
    endpoint: u8,
) -> Result<UsbHolder> {
    for i in 0..device.device_descriptor()?.num_configurations() {
        for interface in device.config_descriptor(i)?.interfaces() {
            if handle.kernel_driver_active(interface.number())? {
                handle.detach_kernel_driver(interface.number())?;
            }

            for descriptor in interface.descriptors() {
                if descriptor.class_code() == 3
                    && descriptor.sub_class_code() == 0
                    && descriptor.protocol_code() == 0
                {
                    // handle.claim_interface(descriptor.interface_number())?;
                }

                for end in descriptor.endpoint_descriptors() {
                    if end.number() == endpoint {
                        return Ok(UsbHolder {
                            device,
                            handle,
                            address: end.address(),
                        });
                    }
                }
            }
        }
    }
    bail!("Invalid address")
}

fn find_usb_device() -> Result<UsbHolder> {
    for device in rusb::devices()?.iter() {
        let descriptor = device.device_descriptor()?;

        if descriptor.vendor_id() != VENDOR_ID {
            continue;
        }

        for (&product, (&endpoint, &index)) in
            PRODUCT_ID.iter().zip(ENDPOINT.iter().zip(INDEX.iter()))
        {
            if descriptor.product_id() != product {
                continue;
            }

            let handle = device.open()?;
            return detach_driver(device, handle, endpoint);
        }

        // println!("Bus {:03} Device {:03} ID {:04x}:{:04x}",
        //          device.bus_number(),
        //          device.address(),
        //          device_desc.vendor_id(),
        //          device_desc.product_id());
    }
    bail!("Device not found")
}

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
struct Coords {
    pub x: f32,
    pub y: f32,
}

fn get_deadzone(gamepad: &Gamepad, axis: Axis) -> Result<f32> {
    gamepad
        .deadzone(gamepad.axis_code(axis).ok_or_eyre("No such axis")?)
        .ok_or_eyre("Can't get a deadzone")
}

fn get_gamepad(gilrs: &Gilrs, id: usize) -> Result<Gamepad> {
    let mut res: Option<Gamepad> = None;
    for (_id, gamepad) in gilrs.gamepads() {
        let _id: usize = _id.into();
        if _id == id {
            res = Some(gamepad);
        }
    }
    res.ok_or_eyre("Couldn't get Gamepad by id")
}

pub fn print_deadzones(gilrs: &Gilrs, id: usize) -> Result<()> {
    let gamepad0 = get_gamepad(gilrs, id)?;
    let mut deadzone = Coords::default();

    deadzone.x = get_deadzone(&gamepad0, Axis::LeftStickX)?;
    deadzone.y = get_deadzone(&gamepad0, Axis::LeftStickY)?;
    println!("Left joystick deadzones: ({}, {})", deadzone.x, deadzone.y);

    deadzone.x = get_deadzone(&gamepad0, Axis::RightStickX)?;
    deadzone.y = get_deadzone(&gamepad0, Axis::RightStickY)?;
    println!("Right joystick deadzones: ({}, {})", deadzone.x, deadzone.y);
    Ok(())
}

fn read_events(
    gilrs: &mut Gilrs,
    shared_info: &SharedInfo,
    configs: &MainConfigs,
    termination_status: &TerminationStatus,
) -> Result<()> {
    let impl_cfg = ImplementationSpecificCfg::new(-1.0, 1.0);

    let input_buffer_refresh_interval = configs.general.input_buffer_refresh_interval;

    // gilrs.next_event().filter_ev()
    print_deadzones(gilrs, 0)?;

    loop {
        if termination_status.check() {
            return Ok(());
        };

        // Examine new events
        while let Some(Event {
                           id, event, time, ..
                       }) = gilrs.next_event()
        {
            let is_disconnected = event == Disconnected;
            debug!("{}", print_event(&event)?);

            let event = normalize_event(&event, shared_info.RESET_BTN)?;
            process_event(event, shared_info, &impl_cfg)?;

            if is_disconnected {
                shared_info.release_all_hard()?;
                println!("Gamepad disconnected");
                return Ok(());
            }
        }
        sleep(input_buffer_refresh_interval);
        // sleep(Duration::from_millis(4)); //4 = USB min latency
    }
}

fn init_gilrs() -> Result<Gilrs> {
    exec_or_eyre!(Gilrs::new())
}

pub fn run_gilrs_loop(
    shared_info: &SharedInfo,
    configs: &MainConfigs,
    termination_status: &TerminationStatus,
) -> Result<()> {
    // let usb_holder = find_usb_device()?;

    let mut gilrs = init_gilrs()?;

    let mut wait_msg_is_printed = false;
    loop {
        gilrs = init_gilrs()?;
        for (id, gamepad) in gilrs.gamepads() {
            println!(
                "id {}: {} is {:?}",
                id,
                gamepad.name(),
                gamepad.power_info()
            );
        }

        match gilrs.gamepads().count() {
            0 => {
                if !wait_msg_is_printed {
                    wait_msg_is_printed = true;
                    println!("Gamepad is not connected. Waiting...");
                }
            }
            1 => {
                println!("Gamepad connected");
                wait_msg_is_printed = false;
                read_events(&mut gilrs, shared_info, configs, termination_status)?;
            }
            _ => {
                println!("Only one gamepad is supported. Disconnect other gamepads");
            }
        }
        sleep(Duration::from_millis(5000));
    }
}

pub fn match_button(code: u16) -> Result<ButtonName> {
    Ok(match code {
        304 => ButtonName::BtnDown_SideR,
        305 => ButtonName::BtnRight_SideR,
        308 => ButtonName::BtnUp_SideR,
        307 => ButtonName::BtnLeft_SideR,
        //
        336 => ButtonName::Wing_SideL,
        337 => ButtonName::Wing_SideR,
        //
        289 => ButtonName::PadAsTouch_SideL,
        290 => ButtonName::PadAsTouch_SideR,
        318 => ButtonName::PadAsBtn_SideR,
        317 => ButtonName::StickAsBtn,
        //
        545 => ButtonName::PadDown_SideL,
        547 => ButtonName::PadRight_SideL,
        544 => ButtonName::PadUp_SideL,
        546 => ButtonName::PadLeft_SideL,
        //
        312 => ButtonName::LowerTriggerAsBtn_SideL,
        313 => ButtonName::LowerTriggerAsBtn_SideR,

        310 => ButtonName::UpperTrigger_SideL,
        311 => ButtonName::UpperTrigger_SideR,
        //
        314 => ButtonName::ExtraBtn_SideL,
        315 => ButtonName::ExtraBtn_SideR,
        316 => ButtonName::ExtraBtnCentral,
        //
        _ => bail!("Unknown button: {code}"),
    })
}

pub fn match_axis(code: u16) -> Result<AxisName> {
    Ok(match code {
        16 => AxisName::PadX_SideL,
        17 => AxisName::PadY_SideL,
        //
        3 => AxisName::PadX_SideR,
        4 => AxisName::PadY_SideR,
        //
        21 => AxisName::LowerTrigger_SideL,
        20 => AxisName::LowerTrigger_SideR,
        //
        0 => AxisName::StickX,
        1 => AxisName::StickY,
        //
        _ => bail!("Unknown button: {code}"),
    })
}

pub fn normalize_event(event: &EventType, RESET_BTN: ButtonName) -> Result<TransformStatus> {
    Ok(match event {
        AxisChanged(axis, value, code) => {
            let code_as_num = print_code(code)?;
            let axis = match_axis(code_as_num)?;
            let value = *value;

            TransformStatus::Transformed(TransformedEvent {
                event_type: EventTypeName::AxisChanged,
                axis,
                //SUPER Important: Steam Controller's Left pad inverts Y axis and thus
                // makes angles negative (angles go clockwise instead of counter-clockwise)
                // need to invert it back
                value: match axis {
                    AxisName::PadY_SideL => -value,
                    _ => value,
                },
                button: ButtonName::None,
            })
        }
        ButtonChanged(button, value, code) => {
            let code_as_num = print_code(code)?;
            TransformStatus::Transformed(TransformedEvent {
                event_type: match *value {
                    0f32 => EventTypeName::ButtonReleased,
                    1f32 => EventTypeName::ButtonPressed,
                    _ => bail!("Cannot happen"),
                },
                axis: AxisName::None,
                value: *value,
                button: match_button(code_as_num)?,
            })
        }
        Disconnected => TransformStatus::Transformed(TransformedEvent {
            event_type: EventTypeName::ButtonReleased,
            axis: AxisName::None,
            button: RESET_BTN,
            value: 0.0,
        }),
        _ => TransformStatus::Discarded,
    })
}

pub fn print_button(button: &Button) -> &str {
    match button {
        Button::South => "South",
        Button::East => "East",
        Button::North => "North",
        Button::West => "West",
        Button::C => "C",
        Button::Z => "Z",
        Button::LeftTrigger => "LeftTrigger",
        Button::LeftTrigger2 => "LeftTrigger2",
        Button::RightTrigger => "RightTrigger",
        Button::RightTrigger2 => "RightTrigger2",
        Button::Select => "Select",
        Button::Start => "Start",
        Button::Mode => "Mode",
        Button::LeftThumb => "LeftThumb",
        Button::RightThumb => "RightThumb",
        Button::DPadUp => "DPadUp",
        Button::DPadDown => "DPadDown",
        Button::DPadLeft => "DPadLeft",
        Button::DPadRight => "DPadRight",
        Button::Unknown => "Unknown",
    }
}

pub fn print_axis(axis: &Axis) -> &str {
    match axis {
        Axis::LeftStickX => "LeftStickX",
        Axis::LeftStickY => "LeftStickY",
        Axis::LeftZ => "LeftZ",
        Axis::RightStickX => "RightStickX",
        Axis::RightStickY => "RightStickY",
        Axis::RightZ => "RightZ",
        Axis::DPadX => "DPadX",
        Axis::DPadY => "DPadY",
        Axis::Unknown => "Unknown",
    }
}

fn print_code(code: &Code) -> Result<u16> {
    let re = Regex::new(r"\(([0-9]+)\)")?;
    let binding = code.to_string();
    let Some(caps) = re.captures(binding.as_str()) else {
        bail!("Can't extract code: {}", code.to_string())
    };
    let code = str::parse::<u16>(&caps[1])?;
    Ok(code)
}

pub fn print_event(event: &EventType) -> Result<String> {
    let mut button_or_axis = "";
    let mut res_value: f32 = 0.0;
    let mut event_type = "";
    let mut code_as_str = String::from("");
    let mut code_as_num: u16 = 0;

    match event {
        AxisChanged(axis, value, code) => {
            event_type = "AxisChanged";
            res_value = *value;
            button_or_axis = print_axis(axis);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonChanged(button, value, code) => {
            event_type = "ButtonChanged";
            res_value = *value;
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonReleased(button, code) => {
            event_type = "ButtonReleased";
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonPressed(button, code) => {
            event_type = "ButtonPressed";
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        ButtonRepeated(button, code) => {
            event_type = "ButtonRepeated";
            button_or_axis = print_button(button);
            code_as_str = code.to_string();
            code_as_num = print_code(code)?;
        }
        Connected => event_type = "Connected",
        Disconnected => event_type = "Disconnected",
        Dropped => event_type = "Dropped",
        ForceFeedbackEffectCompleted => event_type = "ForceFeedbackEffectCompleted",
        _ => {}
    };
    Ok(format!("{event_type}; BtnOrAxis: {button_or_axis}; Value: {:.3}; Code: {code_as_str}; Num: {code_as_num}", res_value))
}
