use std::env::current_dir;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::{Result};
use crate::exec_or_eyre;
use crate::match_event::ButtonName;


const PROJECT_NAME: &str = "JoystickFullRust";

lazy_static! {
    pub static ref CONFIGS_DIR: PathBuf = get_project_dir().join("config");
    pub static ref LAYOUTS_DIR: PathBuf = CONFIGS_DIR.join("layouts");

    pub static ref GLOBAL_CONFIGS: Configs = Configs::load();
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    #[serde(alias = "typing_layout")]
    pub typing_layout_name: String,
    #[serde(alias = "buttons_layout")]
    pub buttons_layout_name: String,
    #[serde(skip)]
    pub buttons_layout: ButtonsLayout,
    #[serde(default)]
    pub finger_rotation: u8,
    #[serde(alias = "triggers_threshold_pct")]
    pub _triggers_threshold_pct: u8,
    #[serde(skip)]
    pub triggers_threshold_f32: f32,
    #[serde(skip)]
    pub channel_size: usize,
    #[serde(skip)]
    pub mouse_interval: Duration,
    pub mouse_speed: u16,
    pub scroll_speed: u16,
    #[serde(alias = "horizontal_threshold_pct")]
    pub _horizontal_threshold_pct: u8,
    #[serde(skip)]
    pub horizontal_threshold_f32: f32,
}

pub fn convert_pct(value: u8) -> f32 {
    value as f32 / 100f32
}

impl Configs {
    pub fn load_raw() -> Result<Configs> {
        read_toml(CONFIGS_DIR.as_path(), "configs")
    }
    pub fn load() -> Configs {
        let mut configs = Self::load_raw().unwrap();
        configs.triggers_threshold_f32 = convert_pct(configs._triggers_threshold_pct);
        configs.horizontal_threshold_f32 = convert_pct(configs._horizontal_threshold_pct);
        configs.channel_size = 100;
        configs.mouse_interval = Duration::from_millis(10);
        configs
    }
}

//FIXME: Remove default
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ButtonsLayout {
    pub gaming_mode: bool,
    #[serde(alias = "switch_button")]
    pub _switch_button: String,
    pub switch_button: ButtonName,
    #[serde(alias = "reset_button")]
    pub _reset_button: String,
    pub reset_button: ButtonName,
}


pub fn last_path_component(path: &Path) -> &str {
    path.components().last().unwrap().as_os_str().to_str().unwrap()
}

pub fn get_project_dir() -> PathBuf {
    let mut cur_dir = current_dir().unwrap();
    while last_path_component(cur_dir.as_path()) != PROJECT_NAME {
        cur_dir = cur_dir.parent().unwrap().to_path_buf();
    }
    cur_dir
}

pub fn read_toml<T, P, S>(folder: P, filename: S) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
        P: AsRef<Path>,
        S: AsRef<str>
{
    const EXTENSION: &str = ".toml";
    let mut filename = filename.as_ref().to_string();
    if !filename.ends_with(EXTENSION) {
        filename += EXTENSION
    }

    let filepath = folder.as_ref().join(filename);
    let file_content = read_to_string(filepath)?;
    let decoded_obj = exec_or_eyre!(toml::from_str(file_content.as_str()));
    decoded_obj
}
