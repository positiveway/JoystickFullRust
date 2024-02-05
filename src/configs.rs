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
use crate::key_codes::KeyCodes;


const PROJECT_NAME: &str = "JoystickFullRust";

lazy_static! {
    pub static ref CONFIGS_DIR: PathBuf = get_project_dir().join("config");
    pub static ref LAYOUTS_DIR: PathBuf = CONFIGS_DIR.join("layouts");
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct JitterThreshold {
    pub left_pad: f32,
    pub right_pad: f32,
    pub stick: f32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FingerRotation {
    pub use_rotation: bool,
    pub left_pad: i16,
    pub right_pad: i16,
    pub stick: i16,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MainConfigs {
    pub debug: bool,
    #[serde(alias = "typing_layout")]
    pub typing_layout_name: String,
    #[serde(alias = "buttons_layout")]
    pub buttons_layout_name: String,
    #[serde(skip)]
    pub channel_size: usize,
    #[serde(skip)]
    pub mouse_interval: Duration,
    #[serde(skip)]
    pub layout_configs: LayoutConfigs,
}

pub fn convert_pct(value: u8) -> f32 {
    value as f32 / 100f32
}

impl MainConfigs {
    pub fn load() -> Result<Self> {
        let mut main_configs: Self = read_toml(CONFIGS_DIR.as_path(), "configs")?;
        main_configs.channel_size = 100;
        main_configs.mouse_interval = Duration::from_millis(1);

        main_configs.layout_configs = LayoutConfigs::load(main_configs.buttons_layout_name.as_str())?;

        Ok(main_configs)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutConfigs {
    pub gaming_mode: bool,
    #[serde(alias = "buttons_layout")]
    _buttons_layout_raw: ButtonsLayoutRaw,
    #[serde(skip)]
    pub buttons_layout: ButtonsLayout,
    pub finger_rotation: FingerRotation,
    #[serde(alias = "WASD_threshold_pct")]
    _wasd_threshold_pct: Option<u8>,
    #[serde(skip)]
    pub wasd_threshold: f32,
    #[serde(alias = "triggers_threshold_pct")]
    _triggers_threshold_pct: u8,
    #[serde(skip)]
    pub triggers_threshold: f32,
    pub mouse_speed: u16,
    pub scroll_speed: u16,
    #[serde(alias = "scroll_horizontal_threshold")]
    _scroll_horizontal_threshold: Option<f32>,
    #[serde(skip)]
    pub scroll_horizontal_threshold: f32,
    pub jitter_threshold: JitterThreshold,
}

impl LayoutConfigs {
    pub fn load<S: AsRef<str>>(layout_name: S) -> Result<Self> {
        let mut layout_configs: Self = read_toml(LAYOUTS_DIR.as_path(), layout_name)?;

        layout_configs.triggers_threshold = convert_pct(layout_configs._triggers_threshold_pct);

        match layout_configs.gaming_mode {
            true => {
                match layout_configs._wasd_threshold_pct {
                    None => {
                        bail!("'wasd_threshold_pct' has to be specified in gaming mode")
                    }
                    Some(wasd_threshold_pct) => {
                        layout_configs.wasd_threshold = convert_pct(wasd_threshold_pct);
                    }
                }
            }
            false => {
                match layout_configs._scroll_horizontal_threshold {
                    None => {
                        bail!("'scroll_horizontal_threshold' has to be specified in desktop mode")
                    }
                    Some(scroll_horizontal_threshold) => {
                        layout_configs.scroll_horizontal_threshold = scroll_horizontal_threshold;
                    }
                }
            }
        }
        layout_configs.buttons_layout = ButtonsLayout::load(layout_configs._buttons_layout_raw.clone())?;

        Ok(layout_configs)
    }
}


pub type Buttons = Vec<Button>;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ButtonsLayout {
    pub reset_btn: ButtonName,
    pub switch_mode_btn: ButtonName,
    //
    pub layout: HashMap<ButtonName, Buttons>,
}

impl ButtonsLayout {
    pub fn load(layout_raw: ButtonsLayoutRaw) -> Result<Self> {
        let mut switch_mode_btn: ButtonName = Default::default();
        let mut reset_btn: ButtonName = Default::default();

        let mut layout: HashMap<ButtonName, Buttons> = HashMap::new();

        let mut key_code_to_button = |button_name: ButtonName, codes: Vec<String>| -> Result<()>{
            let mut buttons = Buttons::new();

            let detect_special = codes.len() == 1;

            for code_as_str in codes {
                let button = KeyCodes::from_config(
                    button_name,
                    code_as_str,
                    &mut reset_btn,
                    &mut switch_mode_btn,
                    detect_special,
                )?;
                buttons.push(button)
            }
            layout.insert(button_name, buttons);

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

        Ok(Self {
            //
            reset_btn,
            switch_mode_btn,
            //
            layout,
        })
    }
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ButtonsLayoutRaw {
    #[serde(default)]
    pub BtnUp_SideL: Vec<String>,
    #[serde(default)]
    pub BtnDown_SideL: Vec<String>,
    #[serde(default)]
    pub BtnLeft_SideL: Vec<String>,
    #[serde(default)]
    pub BtnRight_SideL: Vec<String>,
    //
    #[serde(default)]
    pub BtnUp_SideR: Vec<String>,
    #[serde(default)]
    pub BtnDown_SideR: Vec<String>,
    #[serde(default)]
    pub BtnLeft_SideR: Vec<String>,
    #[serde(default)]
    pub BtnRight_SideR: Vec<String>,
    //
    #[serde(default)]
    pub Wing_SideL: Vec<String>,
    #[serde(default)]
    pub Wing_SideR: Vec<String>,
    //
    #[serde(default)]
    pub LowerTriggerAsBtn_SideL: Vec<String>,
    #[serde(default)]
    pub LowerTriggerAsBtn_SideR: Vec<String>,
    //
    #[serde(default)]
    pub UpperTrigger_SideL: Vec<String>,
    #[serde(default)]
    pub UpperTrigger_SideR: Vec<String>,
    //
    #[serde(default)]
    pub PadAsBtn_SideL: Vec<String>,
    #[serde(default)]
    pub PadAsBtn_SideR: Vec<String>,
    #[serde(default)]
    pub StickAsBtn: Vec<String>,
    //
    #[serde(default)]
    pub PadUp_SideL: Vec<String>,
    #[serde(default)]
    pub PadDown_SideL: Vec<String>,
    #[serde(default)]
    pub PadLeft_SideL: Vec<String>,
    #[serde(default)]
    pub PadRight_SideL: Vec<String>,
    //
    #[serde(default)]
    pub PadUp_SideR: Vec<String>,
    #[serde(default)]
    pub PadDown_SideR: Vec<String>,
    #[serde(default)]
    pub PadLeft_SideR: Vec<String>,
    #[serde(default)]
    pub PadRight_SideR: Vec<String>,
    //
    #[serde(default)]
    pub ExtraBtn_SideL: Vec<String>,
    #[serde(default)]
    pub ExtraBtn_SideR: Vec<String>,
    #[serde(default)]
    pub ExtraBtnCentral: Vec<String>,
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
