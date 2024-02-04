use std::collections::HashMap;
use std::env::current_dir;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use color_eyre::eyre::{bail, Result};
use crate::exec_or_eyre;
use crate::match_event::ButtonName;
use mouse_keyboard_input::Button;
use strum::IntoEnumIterator;
use crate::buttons_state::ButtonsState;
use crate::key_codes::KeyCodes;


const PROJECT_NAME: &str = "JoystickFullRust";

lazy_static! {
    pub static ref CONFIGS_DIR: PathBuf = get_project_dir().join("config");
    pub static ref LAYOUTS_DIR: PathBuf = CONFIGS_DIR.join("layouts");
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JitterThreshold {
    pub left_pad: f32,
    pub right_pad: f32,
    pub stick: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FingerRotation {
    pub use_rotation: bool,
    pub left_pad: i16,
    pub right_pad: i16,
    pub stick: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Configs {
    pub debug: bool,
    #[serde(alias = "typing_layout")]
    pub typing_layout_name: String,
    #[serde(alias = "buttons_layout")]
    pub buttons_layout_name: String,
    #[serde(skip)]
    pub buttons_layout: ButtonsLayout,
    pub finger_rotation: FingerRotation,
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
    pub jitter_threshold: JitterThreshold,
}

pub fn convert_pct(value: u8) -> f32 {
    value as f32 / 100f32
}

impl Configs {
    pub fn load() -> Result<Self> {
        let mut configs: Self = read_toml(CONFIGS_DIR.as_path(), "configs")?;

        configs.triggers_threshold_f32 = convert_pct(configs._triggers_threshold_pct);
        configs.horizontal_threshold_f32 = convert_pct(configs._horizontal_threshold_pct);

        configs.channel_size = 100;
        configs.mouse_interval = Duration::from_millis(1);

        configs.buttons_layout = ButtonsLayout::load(configs.buttons_layout_name.as_str())?;

        Ok(configs)
    }
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ButtonsLayoutRaw {
    pub gaming_mode: bool,
    //
    #[serde(default)]
    pub BtnUp_SideL: String,
    #[serde(default)]
    pub BtnDown_SideL: String,
    #[serde(default)]
    pub BtnLeft_SideL: String,
    #[serde(default)]
    pub BtnRight_SideL: String,
    //
    #[serde(default)]
    pub BtnUp_SideR: String,
    #[serde(default)]
    pub BtnDown_SideR: String,
    #[serde(default)]
    pub BtnLeft_SideR: String,
    #[serde(default)]
    pub BtnRight_SideR: String,
    //
    #[serde(default)]
    pub Wing_SideL: String,
    #[serde(default)]
    pub Wing_SideR: String,
    //
    #[serde(default)]
    pub LowerTriggerAsBtn_SideL: String,
    #[serde(default)]
    pub LowerTriggerAsBtn_SideR: String,
    //
    #[serde(default)]
    pub UpperTrigger_SideL: String,
    #[serde(default)]
    pub UpperTrigger_SideR: String,
    //
    #[serde(default)]
    pub PadAsBtn_SideL: String,
    #[serde(default)]
    pub PadAsBtn_SideR: String,
    #[serde(default)]
    pub StickAsBtn: String,
    //
    #[serde(default)]
    pub PadUp_SideL: String,
    #[serde(default)]
    pub PadDown_SideL: String,
    #[serde(default)]
    pub PadLeft_SideL: String,
    #[serde(default)]
    pub PadRight_SideL: String,
    //
    #[serde(default)]
    pub PadUp_SideR: String,
    #[serde(default)]
    pub PadDown_SideR: String,
    #[serde(default)]
    pub PadLeft_SideR: String,
    #[serde(default)]
    pub PadRight_SideR: String,
    //
    #[serde(default)]
    pub ExtraBtn_SideL: String,
    #[serde(default)]
    pub ExtraBtn_SideR: String,
    #[serde(default)]
    pub ExtraBtnCentral: String,
}

impl ButtonsLayoutRaw {
    pub fn load<S: AsRef<str>>(layout_name: S) -> Result<Self> {
        let layout: Self = read_toml(LAYOUTS_DIR.as_path(), layout_name)?;
        Ok(layout)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ButtonsLayout {
    pub gaming_mode: bool,
    //
    pub reset_btn: ButtonName,
    pub switch_mode_btn: ButtonName,
    //
    pub layout: HashMap<ButtonName, Button>,
}

impl ButtonsLayout {
    pub fn load<S: AsRef<str>>(layout_name: S) -> Result<Self> {
        let layout_raw = ButtonsLayoutRaw::load(layout_name)?;
        let mut switch_mode_btn: ButtonName = Default::default();
        let mut reset_btn: ButtonName = Default::default();

        let mut layout: HashMap<ButtonName, Button> = HashMap::new();

        let mut key_code_to_button = |button_name: ButtonName, code_as_str: String| -> Result<()>{
            let button = KeyCodes::from_config(button_name, code_as_str, &mut reset_btn, &mut switch_mode_btn)?;
            layout.insert(button_name, button);
            Ok(())
        };

        key_code_to_button(ButtonName::BtnUp_SideL, layout_raw.BtnUp_SideL)?;
        key_code_to_button(ButtonName::BtnDown_SideL, layout_raw.BtnDown_SideL)?;
        key_code_to_button(ButtonName::BtnLeft_SideL, layout_raw.BtnLeft_SideL)?;
        key_code_to_button(ButtonName::BtnRight_SideL, layout_raw.BtnRight_SideL)?;
        key_code_to_button(ButtonName::BtnUp_SideR, layout_raw.BtnUp_SideR)?;
        key_code_to_button(ButtonName::BtnDown_SideR, layout_raw.BtnDown_SideR)?;
        key_code_to_button(ButtonName::BtnLeft_SideR, layout_raw.BtnLeft_SideR)?;
        key_code_to_button(ButtonName::BtnRight_SideR, layout_raw.BtnRight_SideR)?;
        key_code_to_button(ButtonName::Wing_SideL, layout_raw.Wing_SideL)?;
        key_code_to_button(ButtonName::Wing_SideR, layout_raw.Wing_SideR)?;
        key_code_to_button(ButtonName::LowerTriggerAsBtn_SideL, layout_raw.LowerTriggerAsBtn_SideL)?;
        key_code_to_button(ButtonName::LowerTriggerAsBtn_SideR, layout_raw.LowerTriggerAsBtn_SideR)?;
        key_code_to_button(ButtonName::UpperTrigger_SideL, layout_raw.UpperTrigger_SideL)?;
        key_code_to_button(ButtonName::UpperTrigger_SideR, layout_raw.UpperTrigger_SideR)?;
        key_code_to_button(ButtonName::PadAsBtn_SideL, layout_raw.PadAsBtn_SideL)?;
        key_code_to_button(ButtonName::PadAsBtn_SideR, layout_raw.PadAsBtn_SideR)?;
        key_code_to_button(ButtonName::StickAsBtn, layout_raw.StickAsBtn)?;
        key_code_to_button(ButtonName::PadUp_SideL, layout_raw.PadUp_SideL)?;
        key_code_to_button(ButtonName::PadDown_SideL, layout_raw.PadDown_SideL)?;
        key_code_to_button(ButtonName::PadLeft_SideL, layout_raw.PadLeft_SideL)?;
        key_code_to_button(ButtonName::PadRight_SideL, layout_raw.PadRight_SideL)?;
        key_code_to_button(ButtonName::PadUp_SideR, layout_raw.PadUp_SideR)?;
        key_code_to_button(ButtonName::PadDown_SideR, layout_raw.PadDown_SideR)?;
        key_code_to_button(ButtonName::PadLeft_SideR, layout_raw.PadLeft_SideR)?;
        key_code_to_button(ButtonName::PadRight_SideR, layout_raw.PadRight_SideR)?;
        key_code_to_button(ButtonName::ExtraBtn_SideL, layout_raw.ExtraBtn_SideL)?;
        key_code_to_button(ButtonName::ExtraBtn_SideR, layout_raw.ExtraBtn_SideR)?;
        key_code_to_button(ButtonName::ExtraBtnCentral, layout_raw.ExtraBtnCentral)?;

        reset_btn.bail_if_not_init()?;
        switch_mode_btn.bail_if_not_init()?;

        Ok(Self {
            gaming_mode: layout_raw.gaming_mode,
            //
            reset_btn,
            switch_mode_btn,
            //
            layout,
        })
    }
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
