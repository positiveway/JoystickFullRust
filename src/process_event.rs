use gilrs::EventType;
use color_eyre::eyre::{Result};
use crate::match_event::*;

pub fn process_event(event: &EventType) -> Result<()> {
    let event = match_event(event)?;
    if event.event_name == EventName::Unknown {
        return Ok(());
    }

    Ok(())
}