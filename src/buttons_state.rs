use std::borrow::Borrow;
use std::collections::HashMap;
use std::hash::Hash;
use color_eyre::eyre::{OptionExt, Result};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
use strum_macros::Display;
use crate::configs::{ButtonsLayout, KeyCodes};
use crate::key_codes::{KeyCode};
use crate::match_event::ButtonName;

#[derive(Display, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Pressed(KeyCode),
    Released(KeyCode),
}

pub type Commands = Vec<Command>;

#[derive(Clone, Debug)]
pub struct ButtonsState {
    pressed: HashMap<KeyCode, bool>,
    RESET_BTN: ButtonName,
    buttons_layout: HashMap<ButtonName, KeyCodes>,
    special_codes: KeyCodes,
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
            KeyCode::None,
            KeyCode::RESET_BTN,
            KeyCode::SWITCH_MODE_BTN,
            KeyCode::RELEASE_ALL,
        ];

        let special_buttons = vec![
            buttons_layout.reset_btn,
            buttons_layout.switch_mode_btn,
        ];

        let mut pressed = HashMap::new();
        for key_code in KeyCode::iter() {
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

    pub fn press_keycodes(&mut self, key_codes: KeyCodes) -> Result<()> {
        if key_codes.len() == 1 {
            let key_code = key_codes[0];
            if key_code == KeyCode::KEY_ESC ||
                key_code == KeyCode::RELEASE_ALL {
                self.release_all()?;
            }
        }
        for key_code in &key_codes {
            if !self.special_codes.contains(key_code) {
                if !*get_or_err(&self.pressed, key_code)? {
                    self.queue.push(Command::Pressed(*key_code));
                    self.pressed.insert(*key_code, true);
                }
            }
        }
        Ok(())
    }

    pub fn release_keycodes(&mut self, key_codes: KeyCodes) -> Result<()> {
        for key_code in key_codes.iter().rev() {
            if !self.special_codes.contains(key_code) {
                if *get_or_err(&self.pressed, key_code)? {
                    self.queue.push(Command::Released(*key_code));
                    self.pressed.insert(*key_code, false);
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

