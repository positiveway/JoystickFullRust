use crate::key_codes::key_code_from_config;
use crate::match_event::ButtonName;
use crate::math_ops::Angle;
use ahash::AHashMap;
use color_eyre::eyre::{bail, OptionExt, Result};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::env::current_dir;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;
use universal_input::{KeyCode, KeyCodes};

const PROJECT_NAME: &str = "JoystickFullRust";

lazy_static! {
    pub static ref CONFIGS_DIR: PathBuf = get_project_dir().join("config");
    pub static ref LAYOUTS_DIR: PathBuf = CONFIGS_DIR.join("layouts");
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct JitterThreshold {
    pub left_pad: f32,
    pub right_pad: f32,
    pub stick: f32,
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct FingerRotation {
    pub use_rotation: bool,
    pub left_pad: i16,
    pub right_pad: i16,
    pub stick: i16,
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct ScrollConfigs {
    pub speed: u16,
    pub horizontal_threshold: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MainConfigs {
    pub debug: bool,
    pub is_steamy: bool,
    #[serde(alias = "typing_layout")]
    pub typing_layout_name: String,
    #[serde(alias = "buttons_layout")]
    pub buttons_layout_name: String,
    #[serde(skip)]
    pub channel_size: usize,
    #[serde(skip)]
    pub usb_input_refresh_interval: Duration,
    #[serde(skip)]
    pub mouse_refresh_interval: Duration,
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
        main_configs.usb_input_refresh_interval = Duration::from_millis(1);
        main_configs.mouse_refresh_interval = Duration::from_millis(1);

        main_configs.layout_configs =
            LayoutConfigs::load(main_configs.buttons_layout_name.as_str())?;

        Ok(main_configs)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ZoneMappingConfigs {
    pub zone_range: Angle,

    #[serde(alias = "start_threshold_pct")]
    _start_threshold_pct: u8,
    #[serde(skip)]
    pub start_threshold: f32,

    #[serde(alias = "shift_threshold_pct")]
    _shift_threshold_pct: Option<u8>,
    #[serde(skip)]
    pub shift_threshold: f32,
    #[serde(skip)]
    pub use_shift: bool,
}

impl ZoneMappingConfigs {
    pub fn load(&mut self) -> Result<()> {
        self.start_threshold = convert_pct(self._start_threshold_pct);

        (self.shift_threshold, self.use_shift) = match self._shift_threshold_pct {
            None => (0.0, false),
            Some(value) => {
                if !(value > 0 && value < 100) {
                    bail!("Incorrect value for 'shift_threshold': '{}'", value);
                }
                (convert_pct(value), true)
            }
        };

        Ok(())
    }

    pub fn load_and_return(&self) -> Result<Self> {
        let mut res = self.clone();
        res.load()?;
        Ok(res)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct GeneralConfigs {
    pub gaming_mode: bool,
    pub repeat_keys: bool,
    #[serde(alias = "triggers_threshold_pct")]
    _triggers_threshold_pct: u8,
    #[serde(skip)]
    pub triggers_threshold: f32,
    pub mouse_speed: u16,
}

impl GeneralConfigs {
    pub fn load(&mut self) -> Result<()> {
        self.triggers_threshold = convert_pct(self._triggers_threshold_pct);
        Ok(())
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct LayoutConfigs {
    #[serde(alias = "ButtonsLayout")]
    _buttons_layout_raw: ButtonsLayoutRaw,
    #[serde(skip)]
    pub buttons_layout: ButtonsLayout,

    #[serde(alias = "General")]
    pub general: GeneralConfigs,

    #[serde(alias = "FingerRotation")]
    pub finger_rotation: FingerRotation,

    #[serde(alias = "Stick")]
    pub stick: ZoneMappingConfigs,

    #[serde(alias = "WASD")]
    _wasd: Option<ZoneMappingConfigs>,
    #[serde(skip)]
    pub wasd: ZoneMappingConfigs,

    #[serde(alias = "Scroll")]
    _scroll: Option<ScrollConfigs>,
    #[serde(skip)]
    pub scroll: ScrollConfigs,

    #[serde(alias = "JitterThreshold")]
    pub jitter_threshold: JitterThreshold,
}

impl LayoutConfigs {
    pub fn load<S: AsRef<str>>(layout_name: S) -> Result<Self> {
        let mut layout_configs: Self = read_toml(LAYOUTS_DIR.as_path(), layout_name)?;
        layout_configs.general.load()?;
        let gaming_mode = layout_configs.general.gaming_mode;

        match gaming_mode {
            true => {
                layout_configs.stick.load()?;

                match layout_configs._wasd {
                    None => {
                        bail!("'Movement' has to be specified in gaming mode")
                    }
                    Some(ref wasd) => {
                        layout_configs.wasd = wasd.load_and_return()?;
                    }
                }
            }
            false => match layout_configs._scroll {
                None => {
                    bail!("'Scroll' has to be specified in desktop mode")
                }
                Some(scroll) => {
                    layout_configs.scroll = scroll;
                }
            },
        }
        layout_configs.buttons_layout = ButtonsLayout::load(
            layout_configs._buttons_layout_raw.clone(),
            layout_configs.general.gaming_mode,
        )?;

        Ok(layout_configs)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ButtonsLayout {
    pub reset_btn: ButtonName,
    pub switch_mode_btn: ButtonName,
    //
    pub layout: AHashMap<ButtonName, KeyCodes>,
}

impl ButtonsLayout {
    pub fn load(layout_raw: ButtonsLayoutRaw, gaming_mode: bool) -> Result<Self> {
        let mut switch_mode_btn = ButtonName::DefaultForSpecialBtns;
        let mut reset_btn = ButtonName::DefaultForSpecialBtns;

        let mut layout: AHashMap<ButtonName, KeyCodes> = AHashMap::new();

        let mut string_to_key_code = |button_name: ButtonName, codes: Vec<String>| -> Result<()> {
            let mut key_codes = KeyCodes::new();

            let detect_special = codes.len() == 1;

            for code_as_str in codes {
                let key_code = key_code_from_config(
                    button_name,
                    code_as_str,
                    &mut reset_btn,
                    &mut switch_mode_btn,
                    detect_special,
                )?;
                key_codes.push(key_code)
            }
            layout.insert(button_name, key_codes);

            Ok(())
        };

        string_to_key_code(ButtonName::BtnUp_SideL, layout_raw.BtnUp_SideL)?;
        string_to_key_code(ButtonName::BtnDown_SideL, layout_raw.BtnDown_SideL)?;
        string_to_key_code(ButtonName::BtnLeft_SideL, layout_raw.BtnLeft_SideL)?;
        string_to_key_code(ButtonName::BtnRight_SideL, layout_raw.BtnRight_SideL)?;
        string_to_key_code(ButtonName::BtnUp_SideR, layout_raw.BtnUp_SideR)?;
        string_to_key_code(ButtonName::BtnDown_SideR, layout_raw.BtnDown_SideR)?;
        string_to_key_code(ButtonName::BtnLeft_SideR, layout_raw.BtnLeft_SideR)?;
        string_to_key_code(ButtonName::BtnRight_SideR, layout_raw.BtnRight_SideR)?;
        string_to_key_code(ButtonName::Wing_SideL, layout_raw.Wing_SideL)?;
        string_to_key_code(ButtonName::Wing_SideR, layout_raw.Wing_SideR)?;
        string_to_key_code(
            ButtonName::LowerTriggerAsBtn_SideL,
            layout_raw.LowerTriggerAsBtn_SideL,
        )?;
        string_to_key_code(
            ButtonName::LowerTriggerAsBtn_SideR,
            layout_raw.LowerTriggerAsBtn_SideR,
        )?;
        string_to_key_code(
            ButtonName::UpperTrigger_SideL,
            layout_raw.UpperTrigger_SideL,
        )?;
        string_to_key_code(
            ButtonName::UpperTrigger_SideR,
            layout_raw.UpperTrigger_SideR,
        )?;
        string_to_key_code(ButtonName::PadAsBtn_SideL, layout_raw.PadAsBtn_SideL)?;
        string_to_key_code(ButtonName::PadAsBtn_SideR, layout_raw.PadAsBtn_SideR)?;
        string_to_key_code(ButtonName::StickAsBtn, layout_raw.StickAsBtn)?;
        string_to_key_code(ButtonName::PadUp_SideL, layout_raw.PadUp_SideL)?;
        string_to_key_code(ButtonName::PadDown_SideL, layout_raw.PadDown_SideL)?;
        string_to_key_code(ButtonName::PadLeft_SideL, layout_raw.PadLeft_SideL)?;
        string_to_key_code(ButtonName::PadRight_SideL, layout_raw.PadRight_SideL)?;
        string_to_key_code(ButtonName::PadUp_SideR, layout_raw.PadUp_SideR)?;
        string_to_key_code(ButtonName::PadDown_SideR, layout_raw.PadDown_SideR)?;
        string_to_key_code(ButtonName::PadLeft_SideR, layout_raw.PadLeft_SideR)?;
        string_to_key_code(ButtonName::PadRight_SideR, layout_raw.PadRight_SideR)?;
        string_to_key_code(ButtonName::ExtraBtn_SideL, layout_raw.ExtraBtn_SideL)?;
        string_to_key_code(ButtonName::ExtraBtn_SideR, layout_raw.ExtraBtn_SideR)?;
        string_to_key_code(ButtonName::ExtraBtnCentral, layout_raw.ExtraBtnCentral)?;

        if !gaming_mode {
            switch_mode_btn.bail_if_special_not_init()?;
            reset_btn.bail_if_special_not_init()?;
        }

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
    path.components()
        .last()
        .unwrap()
        .as_os_str()
        .to_str()
        .unwrap()
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
        S: AsRef<str>,
{
    const EXTENSION: &str = ".toml";
    let mut filename = filename.as_ref().to_string();
    if !filename.ends_with(EXTENSION) {
        filename += EXTENSION
    }

    let filepath = folder.as_ref().join(filename);
    let file_content = read_to_string(filepath)?;
    let decoded_obj = toml::from_str(file_content.as_str())?;
    Ok(decoded_obj)
}
