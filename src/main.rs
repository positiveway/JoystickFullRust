mod deadzones;
mod match_event;
mod shared;
mod configs;
mod process_event;
mod mouse;

use std::thread::sleep;
use std::time::Duration;
use color_eyre::eyre::{Result};
use gilrs::{Event, EventType::*, Gilrs};
use crate::configs::{Configs};
use crate::deadzones::print_deadzones;
use crate::match_event::print_event;
use crate::mouse::{create_writing_thread};
use crate::process_event::{ControllerState, process_event};

static IS_DEBUG: bool = true;

fn read_send_events(gilrs: &mut Gilrs, controller_state: &ControllerState) -> Result<()> {
    // gilrs.next_event().filter_ev()
    print_deadzones(gilrs, 0)?;

    loop {
        // Examine new events
        while let Some(Event { id, event, time }) = gilrs.next_event() {
            process_event(&event, &controller_state)?;

            if IS_DEBUG {
                println!("{}", print_event(&event)?);
            }

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
    let configs = Configs::load()?;
    println!("Layout: {}", configs.buttons_layout_name);

    let mut controller_state = ControllerState::new(configs.clone());
    let thread_handle = create_writing_thread(
        controller_state.mouse_receiver.clone(),
        controller_state.button_receiver.clone(),
        configs,
    );

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
            read_send_events(&mut gilrs, &controller_state)?;
        }
        sleep(Duration::from_millis(5000));
    }

    thread_handle.join().unwrap();
    Ok(())
}

// Don't use lazy_static with multiple threads.
// Lock poisoning or CPU-level contention will occur.
// One thread will stay in locked state
fn main() -> Result<()> {
    color_eyre::install()?;

    init_controller()
}