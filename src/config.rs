use lazy_static::lazy_static;
use serde::Deserialize;


lazy_static! {
    pub static ref GLOBAL_CONFIG: Config = Config::load();
}


#[derive(Deserialize)]
pub struct Config {
    pub typing_layout: String,
    pub buttons_layout: String,
    #[serde(default)]
    pub finger_rotation: u8,
    pub triggers_threshold_pct: u8,
}

impl Config {
    pub fn load() -> Config {
        Config {
            typing_layout: "".to_string(),
            buttons_layout: "".to_string(),
            finger_rotation: 0,
            triggers_threshold_pct: 0,
        }
    }
}
// ButtonsLayout TypingLayout

#[derive(Deserialize)]
pub struct ButtonsLayout {}