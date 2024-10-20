use crate::match_event::ButtonName;
use color_eyre::eyre::{bail, Report, Result};
use universal_input::{KeyCode, KeyCodes};

fn assign_special_button(special_button: &mut ButtonName, value: ButtonName) -> Result<KeyCode> {
    if *special_button != ButtonName::DefaultForSpecialBtns {
        bail!(
            "Duplicate of special button: '{}'. Value already exists: '{}'",
            value,
            *special_button
        )
    } else {
        *special_button = value;
        Ok(KeyCode::None)
    }
}

pub fn key_code_from_config(
    button_name: ButtonName,
    code_str: String,
    reset_btn: &mut ButtonName,
    switch_mode_btn: &mut ButtonName,
    detect_special: bool,
) -> Result<KeyCode> {
    if code_str == "" {
        return Ok(KeyCode::None);
    };

    let key_code = KeyCode::try_from(code_str.as_str());
    match key_code {
        Err(err) => Err(Report::new(err).wrap_err(format!("'{button_name}'"))),
        Ok(key_code) => {
            if detect_special {
                match key_code {
                    KeyCode::RESET_BTN => {
                        return assign_special_button(reset_btn, button_name);
                    }
                    KeyCode::SWITCH_MODE_BTN => {
                        return assign_special_button(switch_mode_btn, button_name);
                    }
                    _ => {}
                }
            }
            Ok(key_code)
        }
    }
}
