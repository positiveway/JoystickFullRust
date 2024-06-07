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
use crate::writing_thread::{create_writing_thread, write_events};
use crate::utils::{check_thread_handle, ThreadHandle, try_unwrap_thread};
use crate::process_event::{process_event, ControllerState};
use color_eyre::eyre::Result;
use env_logger::builder;
use log::debug;
use std::{env, thread};

fn load_configs() -> Result<(ControllerState, MainConfigs)> {
    let configs = MainConfigs::load()?;
    let is_debug = configs.debugging_cfg.is_debug;

    if env::var("RUST_LOG").is_err() {
        env::set_var(
            "RUST_LOG",
            match is_debug {
                true => "debug",
                false => "warn",
            },
        )
    }
    builder()
        .format_module_path(false)
        .format_target(false)
        .format_indent(None)
        .format_timestamp(None)
        .init();

    debug!("Layout: {}", configs.layout_names_cfg.buttons_layout_name);

    let controller_state = ControllerState::new(configs.clone());
    Ok((controller_state, configs))
}

fn init_controller() -> Result<()> {
    println!("App started");

    let (mut controller_state, configs) = load_configs()?;

    let main_as_thread = configs.debugging_cfg.main_as_thread;
    let use_steamy = configs.debugging_cfg.use_steamy;

    let thread_handle = match main_as_thread {
        true => {
            let mouse_receiver = controller_state.mouse_receiver.clone();
            let button_receiver = controller_state.button_receiver.clone();
            let configs_copy = configs.clone();

            let thread_handle = thread::spawn(move || {
                match use_steamy {
                    true => {
                        use crate::steamy_specific::run_steamy_loop;
                        run_steamy_loop(controller_state, configs, None).unwrap();
                    }
                    false => {
                        // use crate::gilrs_specific::run_gilrs_loop;
                        // run_gilrs_loop(controller_state, None).unwrap();
                    }
                };
            });

            write_events(mouse_receiver, button_receiver, configs_copy, Some(&thread_handle))?;

            thread_handle
        }
        false => {
            let thread_handle = create_writing_thread(
                controller_state.mouse_receiver.clone(),
                controller_state.button_receiver.clone(),
                configs.clone(),
            );

            match use_steamy {
                true => {
                    use crate::steamy_specific::run_steamy_loop;
                    run_steamy_loop(controller_state, configs, Some(&thread_handle))?;
                },
                false => {
                    use crate::gilrs_specific::run_gilrs_loop;
                    run_gilrs_loop(controller_state, configs, Some(&thread_handle))?;
                }
            };

            thread_handle
        }
    };

    try_unwrap_thread(thread_handle);

    Ok(())
}


// Don't use lazy_static with multiple threads.
// Lock poisoning or CPU-level contention will occur.
// One thread will stay in locked state
fn main() -> Result<()> {
    color_eyre::install()?;

    init_controller()
}
