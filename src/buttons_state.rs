use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use color_eyre::eyre::{OptionExt, Result};
use mouse_keyboard_input::Button;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;
use crate::configs::{Buttons, ButtonsLayout};
use crate::key_codes::{KEY_CODES_MAX_VALUE, KeyCodes};
use crate::match_event::ButtonName;

#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Pressed(Button),
    Released(Button),
}

pub type Commands = Vec<Command>;

#[derive(Clone, Debug)]
pub struct ButtonsState {
    // pressed: [bool; KEY_CODES_MAX_VALUE],
    pressed: HashMap<Button, bool>,
    RESET_BTN: ButtonName,
    buttons_layout: HashMap<ButtonName, Buttons>,
    special_codes: Buttons,
    special_buttons: Vec<ButtonName>,
    repeat_keys: bool,
    pub queue: Commands,
}

pub fn get_or_err<'a, K: Hash + Eq + Sized + std::fmt::Display, V>(m: &'a HashMap<K, V>, key: &'a K) -> Result<&'a V>
{
    m.get(&key).ok_or_eyre(format!("No mapping for '{}'", &key))
}

impl ButtonsState {
    pub fn new(buttons_layout: ButtonsLayout, repeat_keys: bool) -> Self {
        let special_codes = vec![
            KeyCodes::None as Button,
            KeyCodes::RESET_BTN as Button,
            KeyCodes::SWITCH_MODE_BTN as Button,
            KeyCodes::RELEASE_ALL as Button,
        ];

        let special_buttons = vec![
            buttons_layout.reset_btn,
            buttons_layout.switch_mode_btn,
        ];

        // let mut pressed: [bool; KEY_CODES_MAX_VALUE] = std::array::from_fn(|_| false);
        let mut pressed = HashMap::new();
        for key_code in KeyCodes::iter() {
            let key_code = key_code as Button;
            if !special_codes.contains(&key_code) {
                pressed.insert(key_code, false);
            }
        }


        Self {
            pressed,
            RESET_BTN: buttons_layout.reset_btn,
            buttons_layout: buttons_layout.layout,
            special_codes,
            special_buttons,
            repeat_keys,
            queue: vec![],
        }
    }

    pub fn press_keycodes(&mut self, key_codes: Vec<Button>) -> Result<()> {
        if key_codes.len() == 1 {
            let key_code = key_codes[0];
            if key_code == KeyCodes::KEY_ESC as Button ||
                key_code == KeyCodes::RELEASE_ALL as Button {
                self.release_all()?;
            }
        }
        for key_code in key_codes {
            if !self.special_codes.contains(&key_code) {
                // if !self.pressed[key_code as usize] {
                if !*get_or_err(&self.pressed, &key_code)? {
                    self.queue.push(Command::Pressed(key_code));
                    // self.pressed[key_code as usize] = true;
                    self.pressed.insert(key_code, true);
                }
            }
        }
        Ok(())
    }

    pub fn release_keycodes(&mut self, key_codes: Vec<Button>) -> Result<()> {
        for key_code in key_codes.iter().rev() {
            if !self.special_codes.contains(key_code) {
                let key_code = *key_code;
                // if self.pressed[key_code as usize] {
                if *get_or_err(&self.pressed, &key_code)? {
                    self.queue.push(Command::Released(key_code));
                    // self.pressed[key_code as usize] = false;
                    self.pressed.insert(key_code, false);
                }
            }
        }
        Ok(())
    }

    pub fn press(&mut self, button_name: ButtonName) -> Result<()> {
        if self.special_buttons.contains(&button_name) {
            return Ok(());
        }

        let key_codes = get_or_err(&self.buttons_layout, &button_name)?;
        self.press_keycodes(key_codes.clone())?;

        Ok(())
    }

    fn release_raw(&mut self, button_name: ButtonName) -> Result<()> {
        if self.special_buttons.contains(&button_name) {
            return Ok(());
        }

        let key_codes = get_or_err(&self.buttons_layout, &button_name)?;
        self.release_keycodes(key_codes.clone())?;

        Ok(())
    }

    pub fn release_all(&mut self) -> Result<()> {
        // for key_code in 0..KEY_CODES_MAX_VALUE {
        //     self.release_keycodes(vec![key_code as Button])?;
        // }
        for key_code in self.pressed.clone().keys() {
            self.release_keycodes(vec![*key_code])?;
        }
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

