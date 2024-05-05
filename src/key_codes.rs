use color_eyre::eyre::bail;
use color_eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use crate::exec_or_eyre;
use mouse_keyboard_input::{Button, key_codes};
use crate::configs::KeyCodes;
use crate::match_event::{ButtonName};


#[derive(EnumIter, EnumString, AsRefStr, Display, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum KeyCode {
    None,
    RESET_BTN,
    SWITCH_MODE_BTN,
    RELEASE_ALL,
    KEY_ESC,
    KEY_1,
    KEY_2,
    KEY_3,
    KEY_4,
    KEY_5,
    KEY_6,
    KEY_7,
    KEY_8,
    KEY_9,
    KEY_10,
    KEY_MINUS,
    KEY_EQUAL,
    KEY_BACKSPACE,
    KEY_TAB,
    KEY_Q,
    KEY_W,
    KEY_E,
    KEY_R,
    KEY_T,
    KEY_Y,
    KEY_U,
    KEY_I,
    KEY_O,
    KEY_P,
    KEY_LEFTBRACE,
    KEY_RIGHTBRACE,
    KEY_ENTER,
    KEY_LEFTCTRL,
    KEY_A,
    KEY_S,
    KEY_D,
    KEY_F,
    KEY_G,
    KEY_H,
    KEY_J,
    KEY_K,
    KEY_L,
    KEY_SEMICOLON,
    KEY_APOSTROPHE,
    KEY_GRAVE,
    KEY_LEFTSHIFT,
    KEY_BACKSLASH,
    KEY_Z,
    KEY_X,
    KEY_C,
    KEY_V,
    KEY_B,
    KEY_N,
    KEY_M,
    KEY_COMMA,
    KEY_DOT,
    KEY_SLASH,
    KEY_RIGHTSHIFT,
    KEY_KPASTERISK,
    KEY_LEFTALT,
    KEY_SPACE,
    KEY_CAPSLOCK,
    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,
    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,
    KEY_F9,
    KEY_F10,
    KEY_NUMLOCK,
    KEY_SCROLLLOCK,
    KEY_KP7,
    KEY_KP8,
    KEY_KP9,
    KEY_KPMINUS,
    KEY_KP4,
    KEY_KP5,
    KEY_KP6,
    KEY_KPPLUS,
    KEY_KP1,
    KEY_KP2,
    KEY_KP3,
    KEY_KP0,
    KEY_KPDOT,

    KEY_ZENKAKUHANKAKU,
    KEY_102ND,
    KEY_F11,
    KEY_F12,
    KEY_RO,
    KEY_KATAKANA,
    KEY_HIRAGANA,
    KEY_HENKAN,
    KEY_KATAKANAHIRAGANA,
    KEY_MUHENKAN,
    KEY_KPJPCOMMA,
    KEY_KPENTER,
    KEY_RIGHTCTRL,
    KEY_KPSLASH,
    KEY_SYSRQ,
    KEY_RIGHTALT,
    KEY_LINEFEED,
    KEY_HOME,
    KEY_UP,
    KEY_PAGEUP,
    KEY_LEFT,
    KEY_RIGHT,
    KEY_END,
    KEY_DOWN,
    KEY_PAGEDOWN,
    KEY_INSERT,
    KEY_DELETE,
    KEY_MACRO,
    KEY_MUTE,
    KEY_VOLUMEDOWN,
    KEY_VOLUMEUP,
    KEY_POWER,
    /* SC System Power Down */
    KEY_KPEQUAL,
    KEY_KPPLUSMINUS,
    KEY_PAUSE,
    KEY_SCALE,
    /* AL Compiz Scale  = Expose */

    KEY_KPCOMMA,
    KEY_HANGEUL,
    KEY_HANJA,
    KEY_YEN,
    KEY_LEFTMETA,
    KEY_RIGHTMETA,
    KEY_COMPOSE,

    KEY_STOP,
    /* AC Stop */
    KEY_AGAIN,
    KEY_PROPS,
    /* AC Properties */
    KEY_UNDO,
    /* AC Undo */
    KEY_FRONT,
    KEY_COPY,
    /* AC Copy */
    KEY_OPEN,
    /* AC Open */
    KEY_PASTE,
    /* AC Paste */
    KEY_FIND,
    /* AC Search */
    KEY_CUT,
    /* AC Cut */
    KEY_HELP,
    /* AL Integrated Help Center */
    KEY_MENU,
    /* Menu  = show menu */
    KEY_CALC,
    /* AL Calculator */
    KEY_SETUP,
    KEY_SLEEP,
    /* SC System Sleep */
    KEY_WAKEUP,
    /* System Wake Up */
    KEY_FILE,
    /* AL Local Machine Browser */
    KEY_SENDFILE,
    KEY_DELETEFILE,
    KEY_XFER,
    KEY_PROG1,
    KEY_PROG2,
    KEY_WWW,
    /* AL Internet Browser */
    KEY_MSDOS,
    /* AL Terminal Lock/Screensaver */
    KEY_SCREENLOCK,
    KEY_ROTATE_DISPLAY,
    KEY_CYCLEWINDOWS,
    KEY_MAIL,
    KEY_BOOKMARKS,
    /* AC Bookmarks */
    KEY_COMPUTER,
    KEY_BACK,
    /* AC Back */
    KEY_FORWARD,
    /* AC Forward */
    KEY_CLOSECD,
    KEY_EJECTCD,
    KEY_EJECTCLOSECD,
    KEY_NEXTSONG,
    KEY_PLAYPAUSE,
    KEY_PREVIOUSSONG,
    KEY_STOPCD,
    KEY_RECORD,
    KEY_REWIND,
    KEY_PHONE,
    /* Media Select Telephone */
    KEY_ISO,
    KEY_CONFIG,
    /* AL Consumer Control Configuration */
    KEY_HOMEPAGE,
    /* AC Home */
    KEY_REFRESH,
    /* AC Refresh */
    KEY_EXIT,
    /* AC Exit */
    KEY_MOVE,
    KEY_EDIT,
    KEY_SCROLLUP,
    KEY_SCROLLDOWN,
    KEY_KPLEFTPAREN,
    KEY_KPRIGHTPAREN,
    KEY_NEW,
    /* AC New */
    KEY_REDO,
    /* AC Redo/Repeat */

    //Mouse
    MOUSE_LEFT,
    MOUSE_RIGHT,
    MOUSE_MIDDLE,
    MOUSE_SIDE,
    MOUSE_EXTRA,
    MOUSE_FORWARD,
    MOUSE_BACK,
    MOUSE_TASK,
}


pub fn key_codes_to_buttons(key_codes: &KeyCodes) -> Result<Vec<Button>> {
    let mut buttons = vec![];
    for key_code in key_codes {
        buttons.push(key_code.as_button()?)
    }
    Ok(buttons)
}

fn assign_special_button(special_button: &mut ButtonName, value: ButtonName) -> Result<KeyCode> {
    if *special_button != ButtonName::None {
        bail!("Duplicate of special button: '{}'. Value already exists: '{}'",
            value, *special_button)
    } else {
        *special_button = value;
        Ok(KeyCode::None)
    }
}

impl KeyCode {
    pub fn as_button(&self) -> Result<Button> {
        match self {
            KeyCode::KEY_ESC => Ok(key_codes::KEY_ESC),
            KeyCode::KEY_1 => Ok(key_codes::KEY_1),
            KeyCode::KEY_2 => Ok(key_codes::KEY_2),
            KeyCode::KEY_3 => Ok(key_codes::KEY_3),
            KeyCode::KEY_4 => Ok(key_codes::KEY_4),
            KeyCode::KEY_5 => Ok(key_codes::KEY_5),
            KeyCode::KEY_6 => Ok(key_codes::KEY_6),
            KeyCode::KEY_7 => Ok(key_codes::KEY_7),
            KeyCode::KEY_8 => Ok(key_codes::KEY_8),
            KeyCode::KEY_9 => Ok(key_codes::KEY_9),
            KeyCode::KEY_10 => Ok(key_codes::KEY_10),
            KeyCode::KEY_MINUS => Ok(key_codes::KEY_MINUS),
            KeyCode::KEY_EQUAL => Ok(key_codes::KEY_EQUAL),
            KeyCode::KEY_BACKSPACE => Ok(key_codes::KEY_BACKSPACE),
            KeyCode::KEY_TAB => Ok(key_codes::KEY_TAB),
            KeyCode::KEY_Q => Ok(key_codes::KEY_Q),
            KeyCode::KEY_W => Ok(key_codes::KEY_W),
            KeyCode::KEY_E => Ok(key_codes::KEY_E),
            KeyCode::KEY_R => Ok(key_codes::KEY_R),
            KeyCode::KEY_T => Ok(key_codes::KEY_T),
            KeyCode::KEY_Y => Ok(key_codes::KEY_Y),
            KeyCode::KEY_U => Ok(key_codes::KEY_U),
            KeyCode::KEY_I => Ok(key_codes::KEY_I),
            KeyCode::KEY_O => Ok(key_codes::KEY_O),
            KeyCode::KEY_P => Ok(key_codes::KEY_P),
            KeyCode::KEY_LEFTBRACE => Ok(key_codes::KEY_LEFTBRACE),
            KeyCode::KEY_RIGHTBRACE => Ok(key_codes::KEY_RIGHTBRACE),
            KeyCode::KEY_ENTER => Ok(key_codes::KEY_ENTER),
            KeyCode::KEY_LEFTCTRL => Ok(key_codes::KEY_LEFTCTRL),
            KeyCode::KEY_A => Ok(key_codes::KEY_A),
            KeyCode::KEY_S => Ok(key_codes::KEY_S),
            KeyCode::KEY_D => Ok(key_codes::KEY_D),
            KeyCode::KEY_F => Ok(key_codes::KEY_F),
            KeyCode::KEY_G => Ok(key_codes::KEY_G),
            KeyCode::KEY_H => Ok(key_codes::KEY_H),
            KeyCode::KEY_J => Ok(key_codes::KEY_J),
            KeyCode::KEY_K => Ok(key_codes::KEY_K),
            KeyCode::KEY_L => Ok(key_codes::KEY_L),
            KeyCode::KEY_SEMICOLON => Ok(key_codes::KEY_SEMICOLON),
            KeyCode::KEY_APOSTROPHE => Ok(key_codes::KEY_APOSTROPHE),
            KeyCode::KEY_GRAVE => Ok(key_codes::KEY_GRAVE),
            KeyCode::KEY_LEFTSHIFT => Ok(key_codes::KEY_LEFTSHIFT),
            KeyCode::KEY_BACKSLASH => Ok(key_codes::KEY_BACKSLASH),
            KeyCode::KEY_Z => Ok(key_codes::KEY_Z),
            KeyCode::KEY_X => Ok(key_codes::KEY_X),
            KeyCode::KEY_C => Ok(key_codes::KEY_C),
            KeyCode::KEY_V => Ok(key_codes::KEY_V),
            KeyCode::KEY_B => Ok(key_codes::KEY_B),
            KeyCode::KEY_N => Ok(key_codes::KEY_N),
            KeyCode::KEY_M => Ok(key_codes::KEY_M),
            KeyCode::KEY_COMMA => Ok(key_codes::KEY_COMMA),
            KeyCode::KEY_DOT => Ok(key_codes::KEY_DOT),
            KeyCode::KEY_SLASH => Ok(key_codes::KEY_SLASH),
            KeyCode::KEY_RIGHTSHIFT => Ok(key_codes::KEY_RIGHTSHIFT),
            KeyCode::KEY_KPASTERISK => Ok(key_codes::KEY_KPASTERISK),
            KeyCode::KEY_LEFTALT => Ok(key_codes::KEY_LEFTALT),
            KeyCode::KEY_SPACE => Ok(key_codes::KEY_SPACE),
            KeyCode::KEY_CAPSLOCK => Ok(key_codes::KEY_CAPSLOCK),
            KeyCode::KEY_F1 => Ok(key_codes::KEY_F1),
            KeyCode::KEY_F2 => Ok(key_codes::KEY_F2),
            KeyCode::KEY_F3 => Ok(key_codes::KEY_F3),
            KeyCode::KEY_F4 => Ok(key_codes::KEY_F4),
            KeyCode::KEY_F5 => Ok(key_codes::KEY_F5),
            KeyCode::KEY_F6 => Ok(key_codes::KEY_F6),
            KeyCode::KEY_F7 => Ok(key_codes::KEY_F7),
            KeyCode::KEY_F8 => Ok(key_codes::KEY_F8),
            KeyCode::KEY_F9 => Ok(key_codes::KEY_F9),
            KeyCode::KEY_F10 => Ok(key_codes::KEY_F10),
            KeyCode::KEY_NUMLOCK => Ok(key_codes::KEY_NUMLOCK),
            KeyCode::KEY_SCROLLLOCK => Ok(key_codes::KEY_SCROLLLOCK),
            KeyCode::KEY_KP7 => Ok(key_codes::KEY_KP7),
            KeyCode::KEY_KP8 => Ok(key_codes::KEY_KP8),
            KeyCode::KEY_KP9 => Ok(key_codes::KEY_KP9),
            KeyCode::KEY_KPMINUS => Ok(key_codes::KEY_KPMINUS),
            KeyCode::KEY_KP4 => Ok(key_codes::KEY_KP4),
            KeyCode::KEY_KP5 => Ok(key_codes::KEY_KP5),
            KeyCode::KEY_KP6 => Ok(key_codes::KEY_KP6),
            KeyCode::KEY_KPPLUS => Ok(key_codes::KEY_KPPLUS),
            KeyCode::KEY_KP1 => Ok(key_codes::KEY_KP1),
            KeyCode::KEY_KP2 => Ok(key_codes::KEY_KP2),
            KeyCode::KEY_KP3 => Ok(key_codes::KEY_KP3),
            KeyCode::KEY_KP0 => Ok(key_codes::KEY_KP0),
            KeyCode::KEY_KPDOT => Ok(key_codes::KEY_KPDOT),
            KeyCode::KEY_ZENKAKUHANKAKU => Ok(key_codes::KEY_ZENKAKUHANKAKU),
            KeyCode::KEY_102ND => Ok(key_codes::KEY_102ND),
            KeyCode::KEY_F11 => Ok(key_codes::KEY_F11),
            KeyCode::KEY_F12 => Ok(key_codes::KEY_F12),
            KeyCode::KEY_RO => Ok(key_codes::KEY_RO),
            KeyCode::KEY_KATAKANA => Ok(key_codes::KEY_KATAKANA),
            KeyCode::KEY_HIRAGANA => Ok(key_codes::KEY_HIRAGANA),
            KeyCode::KEY_HENKAN => Ok(key_codes::KEY_HENKAN),
            KeyCode::KEY_KATAKANAHIRAGANA => Ok(key_codes::KEY_KATAKANAHIRAGANA),
            KeyCode::KEY_MUHENKAN => Ok(key_codes::KEY_MUHENKAN),
            KeyCode::KEY_KPJPCOMMA => Ok(key_codes::KEY_KPJPCOMMA),
            KeyCode::KEY_KPENTER => Ok(key_codes::KEY_KPENTER),
            KeyCode::KEY_RIGHTCTRL => Ok(key_codes::KEY_RIGHTCTRL),
            KeyCode::KEY_KPSLASH => Ok(key_codes::KEY_KPSLASH),
            KeyCode::KEY_SYSRQ => Ok(key_codes::KEY_SYSRQ),
            KeyCode::KEY_RIGHTALT => Ok(key_codes::KEY_RIGHTALT),
            KeyCode::KEY_LINEFEED => Ok(key_codes::KEY_LINEFEED),
            KeyCode::KEY_HOME => Ok(key_codes::KEY_HOME),
            KeyCode::KEY_UP => Ok(key_codes::KEY_UP),
            KeyCode::KEY_PAGEUP => Ok(key_codes::KEY_PAGEUP),
            KeyCode::KEY_LEFT => Ok(key_codes::KEY_LEFT),
            KeyCode::KEY_RIGHT => Ok(key_codes::KEY_RIGHT),
            KeyCode::KEY_END => Ok(key_codes::KEY_END),
            KeyCode::KEY_DOWN => Ok(key_codes::KEY_DOWN),
            KeyCode::KEY_PAGEDOWN => Ok(key_codes::KEY_PAGEDOWN),
            KeyCode::KEY_INSERT => Ok(key_codes::KEY_INSERT),
            KeyCode::KEY_DELETE => Ok(key_codes::KEY_DELETE),
            KeyCode::KEY_MACRO => Ok(key_codes::KEY_MACRO),
            KeyCode::KEY_MUTE => Ok(key_codes::KEY_MUTE),
            KeyCode::KEY_VOLUMEDOWN => Ok(key_codes::KEY_VOLUMEDOWN),
            KeyCode::KEY_VOLUMEUP => Ok(key_codes::KEY_VOLUMEUP),
            KeyCode::KEY_POWER => Ok(key_codes::KEY_POWER),
            KeyCode::KEY_KPEQUAL => Ok(key_codes::KEY_KPEQUAL),
            KeyCode::KEY_KPPLUSMINUS => Ok(key_codes::KEY_KPPLUSMINUS),
            KeyCode::KEY_PAUSE => Ok(key_codes::KEY_PAUSE),
            KeyCode::KEY_SCALE => Ok(key_codes::KEY_SCALE),
            KeyCode::KEY_KPCOMMA => Ok(key_codes::KEY_KPCOMMA),
            KeyCode::KEY_HANGEUL => Ok(key_codes::KEY_HANGEUL),
            KeyCode::KEY_HANJA => Ok(key_codes::KEY_HANJA),
            KeyCode::KEY_YEN => Ok(key_codes::KEY_YEN),
            KeyCode::KEY_LEFTMETA => Ok(key_codes::KEY_LEFTMETA),
            KeyCode::KEY_RIGHTMETA => Ok(key_codes::KEY_RIGHTMETA),
            KeyCode::KEY_COMPOSE => Ok(key_codes::KEY_COMPOSE),
            KeyCode::KEY_STOP => Ok(key_codes::KEY_STOP),
            KeyCode::KEY_AGAIN => Ok(key_codes::KEY_AGAIN),
            KeyCode::KEY_PROPS => Ok(key_codes::KEY_PROPS),
            KeyCode::KEY_UNDO => Ok(key_codes::KEY_UNDO),
            KeyCode::KEY_FRONT => Ok(key_codes::KEY_FRONT),
            KeyCode::KEY_COPY => Ok(key_codes::KEY_COPY),
            KeyCode::KEY_OPEN => Ok(key_codes::KEY_OPEN),
            KeyCode::KEY_PASTE => Ok(key_codes::KEY_PASTE),
            KeyCode::KEY_FIND => Ok(key_codes::KEY_FIND),
            KeyCode::KEY_CUT => Ok(key_codes::KEY_CUT),
            KeyCode::KEY_HELP => Ok(key_codes::KEY_HELP),
            KeyCode::KEY_MENU => Ok(key_codes::KEY_MENU),
            KeyCode::KEY_CALC => Ok(key_codes::KEY_CALC),
            KeyCode::KEY_SETUP => Ok(key_codes::KEY_SETUP),
            KeyCode::KEY_SLEEP => Ok(key_codes::KEY_SLEEP),
            KeyCode::KEY_WAKEUP => Ok(key_codes::KEY_WAKEUP),
            KeyCode::KEY_FILE => Ok(key_codes::KEY_FILE),
            KeyCode::KEY_SENDFILE => Ok(key_codes::KEY_SENDFILE),
            KeyCode::KEY_DELETEFILE => Ok(key_codes::KEY_DELETEFILE),
            KeyCode::KEY_XFER => Ok(key_codes::KEY_XFER),
            KeyCode::KEY_PROG1 => Ok(key_codes::KEY_PROG1),
            KeyCode::KEY_PROG2 => Ok(key_codes::KEY_PROG2),
            KeyCode::KEY_WWW => Ok(key_codes::KEY_WWW),
            KeyCode::KEY_MSDOS => Ok(key_codes::KEY_MSDOS),
            KeyCode::KEY_SCREENLOCK => Ok(key_codes::KEY_SCREENLOCK),
            KeyCode::KEY_ROTATE_DISPLAY => Ok(key_codes::KEY_ROTATE_DISPLAY),
            KeyCode::KEY_CYCLEWINDOWS => Ok(key_codes::KEY_CYCLEWINDOWS),
            KeyCode::KEY_MAIL => Ok(key_codes::KEY_MAIL),
            KeyCode::KEY_BOOKMARKS => Ok(key_codes::KEY_BOOKMARKS),
            KeyCode::KEY_COMPUTER => Ok(key_codes::KEY_COMPUTER),
            KeyCode::KEY_BACK => Ok(key_codes::KEY_BACK),
            KeyCode::KEY_FORWARD => Ok(key_codes::KEY_FORWARD),
            KeyCode::KEY_CLOSECD => Ok(key_codes::KEY_CLOSECD),
            KeyCode::KEY_EJECTCD => Ok(key_codes::KEY_EJECTCD),
            KeyCode::KEY_EJECTCLOSECD => Ok(key_codes::KEY_EJECTCLOSECD),
            KeyCode::KEY_NEXTSONG => Ok(key_codes::KEY_NEXTSONG),
            KeyCode::KEY_PLAYPAUSE => Ok(key_codes::KEY_PLAYPAUSE),
            KeyCode::KEY_PREVIOUSSONG => Ok(key_codes::KEY_PREVIOUSSONG),
            KeyCode::KEY_STOPCD => Ok(key_codes::KEY_STOPCD),
            KeyCode::KEY_RECORD => Ok(key_codes::KEY_RECORD),
            KeyCode::KEY_REWIND => Ok(key_codes::KEY_REWIND),
            KeyCode::KEY_PHONE => Ok(key_codes::KEY_PHONE),
            KeyCode::KEY_ISO => Ok(key_codes::KEY_ISO),
            KeyCode::KEY_CONFIG => Ok(key_codes::KEY_CONFIG),
            KeyCode::KEY_HOMEPAGE => Ok(key_codes::KEY_HOMEPAGE),
            KeyCode::KEY_REFRESH => Ok(key_codes::KEY_REFRESH),
            KeyCode::KEY_EXIT => Ok(key_codes::KEY_EXIT),
            KeyCode::KEY_MOVE => Ok(key_codes::KEY_MOVE),
            KeyCode::KEY_EDIT => Ok(key_codes::KEY_EDIT),
            KeyCode::KEY_SCROLLUP => Ok(key_codes::KEY_SCROLLUP),
            KeyCode::KEY_SCROLLDOWN => Ok(key_codes::KEY_SCROLLDOWN),
            KeyCode::KEY_KPLEFTPAREN => Ok(key_codes::KEY_KPLEFTPAREN),
            KeyCode::KEY_KPRIGHTPAREN => Ok(key_codes::KEY_KPRIGHTPAREN),
            KeyCode::KEY_NEW => Ok(key_codes::KEY_NEW),
            KeyCode::KEY_REDO => Ok(key_codes::KEY_REDO),
            KeyCode::MOUSE_LEFT => Ok(key_codes::BTN_LEFT),
            KeyCode::MOUSE_RIGHT => Ok(key_codes::BTN_RIGHT),
            KeyCode::MOUSE_MIDDLE => Ok(key_codes::BTN_MIDDLE),
            KeyCode::MOUSE_SIDE => Ok(key_codes::BTN_SIDE),
            KeyCode::MOUSE_EXTRA => Ok(key_codes::BTN_EXTRA),
            KeyCode::MOUSE_FORWARD => Ok(key_codes::BTN_FORWARD),
            KeyCode::MOUSE_BACK => Ok(key_codes::BTN_BACK),
            KeyCode::MOUSE_TASK => Ok(key_codes::BTN_TASK),
            value => bail!("No such key code: {value}")
        }
    }

    pub fn from_config(
        button_name: ButtonName,
        code_str: String,
        reset_btn: &mut ButtonName,
        switch_mode_btn: &mut ButtonName,
        detect_special: bool,
    ) -> Result<Self>
    {
        if code_str == "" {
            return Ok(Self::None);
        };

        let key_code = Self::try_from(code_str.as_str());
        match key_code {
            Err(err) => {
                Err(Report::new(err).wrap_err(format!("'{button_name}'")))
            }
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
}