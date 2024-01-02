use lazy_static::lazy_static;
use serde::Deserialize;


lazy_static! {
    pub static ref GLOBAL_CONFIG: Config = Config::load();
}
// pub static GLOBAL_CONFIG: Config = Config{
//     typing_layout: "".to_string(),
//     buttons_layout: "".to_string(),
//     finger_rotation: None,
//     triggers_threshold_pct: 0,
// };


#[derive(Deserialize)]
pub struct Config {
    pub typing_layout: String,
    pub buttons_layout: String,
    pub finger_rotation: Option<u8>,
    pub triggers_threshold_pct: u8,
}

impl Config {
    pub fn load() -> Config {
        Config {
            typing_layout: "".to_string(),
            buttons_layout: "".to_string(),
            finger_rotation: None,
            triggers_threshold_pct: 0,
        }
    }
}
// ButtonsLayout TypingLayout

#[derive(Deserialize)]
pub struct ButtonsLayout {}