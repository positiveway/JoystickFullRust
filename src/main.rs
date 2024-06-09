#![feature(const_try)]

mod buttons_state;
mod configs;
mod gilrs_specific;
mod key_codes;
mod match_event;
mod math_ops;
mod process_event;
mod steamy_debug;
mod steamy_event;
mod steamy_specific;
mod steamy_state;
mod utils;
mod writing_thread;
mod pads_ops;
mod file_ops;

use crate::configs::MainConfigs;
use crate::writing_thread::{write_events};
use crate::utils::{TerminationStatus, ThreadHandle};
use crate::process_event::{process_event, ControllerState};
use color_eyre::eyre::Result;
use env_logger::builder;
use log::debug;
use std::{env, thread};

fn init_logger() {
    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            {
                #[cfg(feature = "debug_mode")] {
                    "debug"
                }
                #[cfg(not(feature = "debug_mode"))]{
                    "warn"
                }
            },
        )
    }
    builder()
        .format_module_path(false)
        .format_target(false)
        .format_indent(None)
        .format_timestamp(None)
        .init();
}

fn load_configs() -> Result<(ControllerState, MainConfigs)> {
    let mut configs = MainConfigs::load()?;

    debug!("Layout: {}", configs.layout_names_cfg.buttons_layout_name);

    let controller_state = ControllerState::new(configs.clone());
    Ok((controller_state, configs))
}

fn init_controller() -> Result<()> {
    println!("App started");

    init_logger();

    let (mut controller_state, configs) = load_configs()?;

    let termination_status = TerminationStatus::default();

    let termination_status_copy = termination_status.clone();
    let termination_status_copy2 = termination_status.clone();
    let mouse_receiver = controller_state.mouse_receiver.clone();
    let button_receiver = controller_state.button_receiver.clone();
    let configs_copy = configs.clone();
    let configs_copy2 = configs.clone();

    #[cfg(not(feature = "main_as_thread"))]{
        termination_status.spawn_with_check(
            move || -> Result<()>{
                write_events(
                    mouse_receiver,
                    button_receiver,
                    configs_copy2,
                    termination_status_copy2,
                )
            }
        );

        termination_status.run_with_check(
            move || {
                #[cfg(feature = "use_steamy")] {
                    use crate::steamy_specific::run_steamy_loop;
                    crate::steamy_specific::run_steamy_loop(controller_state, configs_copy, termination_status_copy)
                }
                #[cfg(not(feature = "use_steamy"))]{
                    use crate::gilrs_specific::run_gilrs_loop;
                    run_gilrs_loop(controller_state, configs_copy, termination_status_copy)
                }
            }
        );
    };

    #[cfg(feature = "main_as_thread")] {
        termination_status.spawn_with_check(
            move || {
                #[cfg(feature = "use_steamy")] {
                    use crate::steamy_specific::run_steamy_loop;
                    crate::steamy_specific::run_steamy_loop(controller_state, configs_copy, termination_status_copy)
                }
                #[cfg(not(feature = "use_steamy"))]{
                    use crate::gilrs_specific::run_gilrs_loop;
                    run_gilrs_loop(controller_state, configs_copy, termination_status_copy)
                }
            }
        );

        termination_status.run_with_check(
            move || -> Result<()>{
                crate::writing_thread::write_events(
                    mouse_receiver,
                    button_receiver,
                    configs_copy2,
                    termination_status_copy2,
                )
            }
        );
    };

    Ok(())
}


// Don't use lazy_static with multiple threads.
// Lock poisoning or CPU-level contention will occur.
// One thread will stay in locked state
fn main() -> Result<()> {
    color_eyre::install()?;

    init_controller()
}
