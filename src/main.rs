mod deadzones;
mod match_events;
mod shared;

use std::thread::sleep;
use std::time::Duration;
use color_eyre::eyre::{Result, eyre};
use gilrs::{Event, EventType::*, Gilrs};
use crate::deadzones::print_deadzones;
use crate::shared::*;
use crate::match_events::match_event;

fn read_send_events(gilrs: &mut Gilrs) -> Result<()> {
    print_deadzones(gilrs, 0)?;

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            let (button_or_axis, res_value, event_type, code) = match_event(&event);

            let event_as_str = format!("Event type: {event_type}; BtnOrAxis: {button_or_axis}; Value: {res_value}; Code: {code}");
            println!("{}", &event_as_str);

            if event == Disconnected {
                println!("Gamepad disconnected");
                return Ok(());
            }
        }
        sleep(Duration::from_millis(4)); //4 = USB min latency
    }
}


fn init_gilrs() -> Result<Gilrs> {
    exec_or_eyre!(Gilrs::new())
}

fn init_controller() -> Result<()> {
    let mut gilrs = init_gilrs()?;

    let mut is_wait_msg_printed = false;
    loop {
        gilrs = init_gilrs()?;
        let mut gamepads_counter = 0;
        for (id, gamepad) in gilrs.gamepads() {
            gamepads_counter += 1;
            println!("id {}: {} is {:?}", id, gamepad.name(), gamepad.power_info());
        }

        if gamepads_counter == 0 {
            if !is_wait_msg_printed {
                is_wait_msg_printed = true;
                println!("Gamepad is not connected. Waiting...");
            }
        } else if gamepads_counter > 1 {
            println!("Only one gamepad is supported. Disconnect other gamepads");
        } else {
            is_wait_msg_printed = false;
            read_send_events(&mut gilrs)?;
        }
        sleep(Duration::from_millis(5000));
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    init_controller()
}