mod match_event;
mod utils;
mod configs;
mod process_event;
mod mouse;
mod math_ops;
mod key_codes;
mod buttons_state;
mod gilrs_specific;
mod steamy_specific;
mod steamy_state;
mod steamy_event;
mod steamy_debug;

use std::env;
use color_eyre::eyre::{Result};
use env_logger::{builder};
use log::debug;
use crate::configs::{MainConfigs};
use crate::mouse::{create_writing_thread};
use crate::process_event::{ControllerState, process_event};

fn init_controller() -> Result<()> {
    let (mut controller_state, configs) = load_configs()?;

    let thread_handle = create_writing_thread(
        controller_state.mouse_receiver.clone(),
        controller_state.button_receiver.clone(),
        configs.clone(),
    );
    match configs.is_steamy {
        true => {
            use crate::steamy_specific::run_steamy_loop;
            run_steamy_loop(controller_state, configs)?;
        }
        false => {
            // use crate::gilrs_specific::run_gilrs_loop;
            // run_gilrs_loop(controller_state)?;
        }
    }

    Ok(())
}

fn load_configs() -> Result<(ControllerState, MainConfigs)> {
    let configs = MainConfigs::load()?;

    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", match configs.debug {
            true => { "debug" }
            false => { "warn" }
        })
    }
    builder()
        .format_module_path(false)
        .format_target(false)
        .format_indent(None)
        .format_timestamp(None)
        .init();

    debug!("Layout: {}", configs.buttons_layout_name);

    let controller_state = ControllerState::new(configs.clone());
    Ok((controller_state, configs))
}

// Don't use lazy_static with multiple threads.
// Lock poisoning or CPU-level contention will occur.
// One thread will stay in locked state
fn main() -> Result<()> {
    color_eyre::install()?;

    init_controller()
}