use color_eyre::eyre::bail;
use color_eyre::{Report, Result};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use crate::exec_or_eyre;
use mouse_keyboard_input::Button;
use crate::match_event::{ButtonName};

pub const KEY_CODES_MAX_VALUE: usize = 550;


#[derive(EnumIter, EnumString, AsRefStr, Display, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum KeyCodes {
    None = 0,
    RESET_BTN = 500,
    SWITCH_MODE_BTN = 501,
    RELEASE_ALL = 502,
    KEY_ESC = 1,
    KEY_1 = 2,
    KEY_2 = 3,
    KEY_3 = 4,
    KEY_4 = 5,
    KEY_5 = 6,
    KEY_6 = 7,
    KEY_7 = 8,
    KEY_8 = 9,
    KEY_9 = 10,
    KEY_10 = 11,
    KEY_MINUS = 12,
    KEY_EQUAL = 13,
    KEY_BACKSPACE = 14,
    KEY_TAB = 15,
    KEY_Q = 16,
    KEY_W = 17,
    KEY_E = 18,
    KEY_R = 19,
    KEY_T = 20,
    KEY_Y = 21,
    KEY_U = 22,
    KEY_I = 23,
    KEY_O = 24,
    KEY_P = 25,
    KEY_LEFTBRACE = 26,
    KEY_RIGHTBRACE = 27,
    KEY_ENTER = 28,
    KEY_LEFTCTRL = 29,
    KEY_A = 30,
    KEY_S = 31,
    KEY_D = 32,
    KEY_F = 33,
    KEY_G = 34,
    KEY_H = 35,
    KEY_J = 36,
    KEY_K = 37,
    KEY_L = 38,
    KEY_SEMICOLON = 39,
    KEY_APOSTROPHE = 40,
    KEY_GRAVE = 41,
    KEY_LEFTSHIFT = 42,
    KEY_BACKSLASH = 43,
    KEY_Z = 44,
    KEY_X = 45,
    KEY_C = 46,
    KEY_V = 47,
    KEY_B = 48,
    KEY_N = 49,
    KEY_M = 50,
    KEY_COMMA = 51,
    KEY_DOT = 52,
    KEY_SLASH = 53,
    KEY_RIGHTSHIFT = 54,
    KEY_KPASTERISK = 55,
    KEY_LEFTALT = 56,
    KEY_SPACE = 57,
    KEY_CAPSLOCK = 58,
    KEY_F1 = 59,
    KEY_F2 = 60,
    KEY_F3 = 61,
    KEY_F4 = 62,
    KEY_F5 = 63,
    KEY_F6 = 64,
    KEY_F7 = 65,
    KEY_F8 = 66,
    KEY_F9 = 67,
    KEY_F10 = 68,
    KEY_NUMLOCK = 69,
    KEY_SCROLLLOCK = 70,
    KEY_KP7 = 71,
    KEY_KP8 = 72,
    KEY_KP9 = 73,
    KEY_KPMINUS = 74,
    KEY_KP4 = 75,
    KEY_KP5 = 76,
    KEY_KP6 = 77,
    KEY_KPPLUS = 78,
    KEY_KP1 = 79,
    KEY_KP2 = 80,
    KEY_KP3 = 81,
    KEY_KP0 = 82,
    KEY_KPDOT = 83,

    KEY_ZENKAKUHANKAKU = 85,
    KEY_102ND = 86,
    KEY_F11 = 87,
    KEY_F12 = 88,
    KEY_RO = 89,
    KEY_KATAKANA = 90,
    KEY_HIRAGANA = 91,
    KEY_HENKAN = 92,
    KEY_KATAKANAHIRAGANA = 93,
    KEY_MUHENKAN = 94,
    KEY_KPJPCOMMA = 95,
    KEY_KPENTER = 96,
    KEY_RIGHTCTRL = 97,
    KEY_KPSLASH = 98,
    KEY_SYSRQ = 99,
    KEY_RIGHTALT = 100,
    KEY_LINEFEED = 101,
    KEY_HOME = 102,
    KEY_UP = 103,
    KEY_PAGEUP = 104,
    KEY_LEFT = 105,
    KEY_RIGHT = 106,
    KEY_END = 107,
    KEY_DOWN = 108,
    KEY_PAGEDOWN = 109,
    KEY_INSERT = 110,
    KEY_DELETE = 111,
    KEY_MACRO = 112,
    KEY_MUTE = 113,
    KEY_VOLUMEDOWN = 114,
    KEY_VOLUMEUP = 115,
    KEY_POWER = 116,
    /* SC System Power Down */
    KEY_KPEQUAL = 117,
    KEY_KPPLUSMINUS = 118,
    KEY_PAUSE = 119,
    KEY_SCALE = 120,
    /* AL Compiz Scale  = Expose */

    KEY_KPCOMMA = 121,
    KEY_HANGEUL = 122,
    KEY_HANJA = 123,
    KEY_YEN = 124,
    KEY_LEFTMETA = 125,
    KEY_RIGHTMETA = 126,
    KEY_COMPOSE = 127,

    KEY_STOP = 128,
    /* AC Stop */
    KEY_AGAIN = 129,
    KEY_PROPS = 130,
    /* AC Properties */
    KEY_UNDO = 131,
    /* AC Undo */
    KEY_FRONT = 132,
    KEY_COPY = 133,
    /* AC Copy */
    KEY_OPEN = 134,
    /* AC Open */
    KEY_PASTE = 135,
    /* AC Paste */
    KEY_FIND = 136,
    /* AC Search */
    KEY_CUT = 137,
    /* AC Cut */
    KEY_HELP = 138,
    /* AL Integrated Help Center */
    KEY_MENU = 139,
    /* Menu  = show menu */
    KEY_CALC = 140,
    /* AL Calculator */
    KEY_SETUP = 141,
    KEY_SLEEP = 142,
    /* SC System Sleep */
    KEY_WAKEUP = 143,
    /* System Wake Up */
    KEY_FILE = 144,
    /* AL Local Machine Browser */
    KEY_SENDFILE = 145,
    KEY_DELETEFILE = 146,
    KEY_XFER = 147,
    KEY_PROG1 = 148,
    KEY_PROG2 = 149,
    KEY_WWW = 150,
    /* AL Internet Browser */
    KEY_MSDOS = 151,
    /* AL Terminal Lock/Screensaver */
    KEY_SCREENLOCK = 152,
    KEY_ROTATE_DISPLAY = 153,
    KEY_CYCLEWINDOWS = 154,
    KEY_MAIL = 155,
    KEY_BOOKMARKS = 156,
    /* AC Bookmarks */
    KEY_COMPUTER = 157,
    KEY_BACK = 158,
    /* AC Back */
    KEY_FORWARD = 159,
    /* AC Forward */
    KEY_CLOSECD = 160,
    KEY_EJECTCD = 161,
    KEY_EJECTCLOSECD = 162,
    KEY_NEXTSONG = 163,
    KEY_PLAYPAUSE = 164,
    KEY_PREVIOUSSONG = 165,
    KEY_STOPCD = 166,
    KEY_RECORD = 167,
    KEY_REWIND = 168,
    KEY_PHONE = 169,
    /* Media Select Telephone */
    KEY_ISO = 170,
    KEY_CONFIG = 171,
    /* AL Consumer Control Configuration */
    KEY_HOMEPAGE = 172,
    /* AC Home */
    KEY_REFRESH = 173,
    /* AC Refresh */
    KEY_EXIT = 174,
    /* AC Exit */
    KEY_MOVE = 175,
    KEY_EDIT = 176,
    KEY_SCROLLUP = 177,
    KEY_SCROLLDOWN = 178,
    KEY_KPLEFTPAREN = 179,
    KEY_KPRIGHTPAREN = 180,
    KEY_NEW = 181,
    /* AC New */
    KEY_REDO = 182,
    /* AC Redo/Repeat */

    //Mouse
    MOUSE_LEFT = 0x110,
    MOUSE_RIGHT = 0x111,
    MOUSE_MIDDLE = 0x112,
    MOUSE_SIDE = 0x113,
    MOUSE_EXTRA = 0x114,
    MOUSE_FORWARD = 0x115,
    MOUSE_BACK = 0x116,
    MOUSE_TASK = 0x117,
}


pub fn key_codes_to_buttons(key_codes: &Vec<KeyCodes>) -> Vec<Button> {
    let buttons: Vec<_> = key_codes.iter().map(|key_code| *key_code as Button).collect();
    buttons
}

fn assign_special_button(special_button: &mut ButtonName, value: ButtonName) -> Result<(Button)> {
    if *special_button != ButtonName::default() {
        bail!("Duplicate of special button: '{}'. Value already exists: '{}'",
            value, *special_button)
    } else {
        *special_button = value;
        Ok(KeyCodes::None as Button)
    }
}

impl KeyCodes {
    // pub fn as_button(&self) -> Button {
    //     *self as Button
    // }

    pub fn from_config(
        button_name: ButtonName,
        code_str: String,
        reset_btn: &mut ButtonName,
        switch_mode_btn: &mut ButtonName,
        detect_special: bool,
    ) -> Result<Button>
    {
        if code_str == "" {
            return Ok(Self::None as Button);
        };

        let key_code = Self::try_from(code_str.as_str());
        match key_code {
            Err(err) => {
                Err(Report::new(err).wrap_err(format!("'{button_name}'")))
            }
            Ok(key_code) => {
                if detect_special {
                    match key_code {
                        KeyCodes::RESET_BTN => {
                            return assign_special_button(reset_btn, button_name);
                        }
                        KeyCodes::SWITCH_MODE_BTN => {
                            return assign_special_button(switch_mode_btn, button_name);
                        }
                        _ => {}
                    }
                }
                Ok(key_code as Button)
            }
        }
    }
}