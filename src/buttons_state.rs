use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use color_eyre::eyre::{OptionExt, Result};
use mouse_keyboard_input::Button;
use strum::IntoEnumIterator;
use crate::configs::{Buttons, ButtonsLayout};
use crate::key_codes::KeyCodes;
use crate::match_event::ButtonName;
use crate::process_event::ButtonEvent::{Pressed, Released};
use crate::process_event::ButtonSender;

#[derive(Clone, Debug)]
pub struct ButtonsState {
    pressed: HashMap<ButtonName, bool>,
    RESET_BTN: ButtonName,
    buttons_layout: HashMap<ButtonName, Buttons>,
    button_sender: ButtonSender,
    special_buttons: Buttons
}

pub fn get_or_err<'a, K: Hash + Eq + Sized + std::fmt::Display, V>(m: &'a HashMap<K, V>, key: &'a K) -> Result<&'a V>
{
    m.get(&key).ok_or_eyre(format!("No mapping for '{}'", &key))
}

impl ButtonsState {
    pub fn new(buttons_layout: ButtonsLayout, button_sender: ButtonSender) -> Self {
        let mut pressed: HashMap<ButtonName, bool> = HashMap::new();
        for button_name in ButtonName::iter() {
            match button_name {
                ButtonName::PadAsTouch_SideL => {}
                ButtonName::PadAsTouch_SideR => {}
                ButtonName::None => {}
                _ => {
                    pressed.insert(button_name, false);
                }
            }
        }
        Self {
            pressed,
            RESET_BTN: buttons_layout.reset_btn,
            buttons_layout: buttons_layout.layout,
            button_sender,
            special_buttons: vec![
                KeyCodes::None as Button,
                KeyCodes::RESET_BTN as Button,
                KeyCodes::SWITCH_MODE_BTN as Button,
                KeyCodes::RELEASE_ALL as Button,
            ]
        }
    }

    pub fn press(&mut self, button_name: ButtonName) -> Result<()> {
        if button_name == self.RESET_BTN {
            return Ok(());
        };

        if !*get_or_err(&self.pressed, &button_name)? {
            self.pressed.insert(button_name, true);
            let key_codes = get_or_err(&self.buttons_layout, &button_name)?.clone();

            if key_codes.len() == 1 {
                let key_code = key_codes[0];
                if key_code == KeyCodes::KEY_ESC as Button ||
                    key_code == KeyCodes::RELEASE_ALL as Button {
                    self.release_all()?;
                }
            }
            for key_code in key_codes {
                if !self.special_buttons.contains(&key_code) {
                    self.button_sender.send(Pressed(key_code))?;
                }
            }
        };
        Ok(())
    }

    fn release_raw(&mut self, button_name: ButtonName) -> Result<()> {
        if *get_or_err(&self.pressed, &button_name)? {
            self.pressed.insert(button_name, false);
            let key_codes = get_or_err(&self.buttons_layout, &button_name)?;
            for key_code in key_codes.iter().rev() {
                if !self.special_buttons.contains(key_code) {
                    self.button_sender.send(Released(*key_code))?;
                }
            }
        };
        Ok(())
    }

    pub fn release_all(&mut self) -> Result<()> {
        for button_name in self.pressed.clone().keys() {
            self.release_raw(*button_name)?;
        };
        Ok(())
    }

    pub fn release(&mut self, button_name: ButtonName) -> Result<()> {
        if button_name == self.RESET_BTN {
            self.release_all()?;
            return Ok(());
        };
        self.release_raw(button_name)?;
        Ok(())
    }
}

