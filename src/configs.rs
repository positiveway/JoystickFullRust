use std::env::current_dir;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use lazy_static::lazy_static;
use serde::Deserialize;
use color_eyre::eyre::{Result};
use crate::exec_or_eyre;


const PROJECT_NAME: &str = "JoystickFullRust";

lazy_static! {
    pub static ref CONFIGS_DIR: PathBuf = {get_project_dir().join("config")};
    pub static ref LAYOUTS_DIR: PathBuf = {CONFIGS_DIR.join("layouts")};

    pub static ref GLOBAL_CONFIGS: Configs = {Configs::load().unwrap()};
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
    if !filename.ends_with(EXTENSION){
        filename += EXTENSION
    }

    let filepath = folder.as_ref().join(filename);
    let file_content = read_to_string(filepath)?;
    let decoded_obj = exec_or_eyre!(toml::from_str(file_content.as_str()));
    decoded_obj
}

#[derive(Deserialize)]
pub struct Configs {
    pub typing_layout: String,
    pub buttons_layout: String,
    #[serde(default)]
    pub finger_rotation: u8,
    pub triggers_threshold_pct: u8,
}

impl Configs {
    pub fn load() -> Result<Configs> {
        println!("reading configs");
        read_toml(CONFIGS_DIR.as_path(), "configs")
    }
}
// ButtonsLayout TypingLayout

#[derive(Deserialize)]
pub struct ButtonsLayout {}