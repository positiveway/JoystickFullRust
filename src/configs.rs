use crate::key_codes::key_code_from_config;
use crate::match_event::ButtonName;
use crate::math_ops::{Angle, coord_to_f32};
use ahash::AHashMap;
use color_eyre::eyre::{bail, OptionExt, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;
use universal_input::{KeyCode, KeyCodes};
use crate::file_ops::{get_project_dir, read_toml};
use crate::steamy_state::SteamyInputCoord;

const PROJECT_NAME: &str = "JoystickFullRust";


#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct JitterThresholdConfigs {
    pub left_pad: f32,
    pub right_pad: f32,
    pub stick: f32,
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct FingerRotationConfigs {
    pub use_rotation: bool,
    pub left_pad: i16,
    pub right_pad: i16,
    pub stick: i16,
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct AxisCorrection {
    #[serde(alias = "x")]
    pub _x: i16,
    #[serde(skip)]
    pub x: f32,

    #[serde(alias = "y")]
    pub _y: i16,
    #[serde(skip)]
    pub y: f32,
}

impl AxisCorrection {
    pub fn clear(&mut self) {
        self._x = 0;
        self.x = 0.0;
        self._y = 0;
        self.y = 0.0;
    }

    pub fn load(&mut self) {
        self.x = coord_to_f32(self._x);
        self.y = coord_to_f32(self._y);
    }
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct AxisCorrectionConfigs {
    pub use_correction: bool,
    pub left_pad: AxisCorrection,
    pub right_pad: AxisCorrection,
    pub stick: AxisCorrection,
}

impl AxisCorrectionConfigs {
    pub fn load(&mut self) {
        if !self.use_correction {
            self.left_pad.clear();
            self.right_pad.clear();
            self.stick.clear();
        }
        self.left_pad.load();
        self.right_pad.load();
        self.stick.load();
    }
}

#[derive(Clone, Debug, Copy, Default, Serialize, Deserialize)]
pub struct ScrollConfigs {
    pub speed: u16,
    pub horizontal_threshold: f32,
}


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MainGeneralConfigs {
    pub commands_channel_size: usize,
    pub steamy_channel_size: usize,

    #[serde(alias = "steamy_read_interrupt_interval")]
    pub _steamy_read_interrupt_interval: u16,
    #[serde(skip)]
    pub steamy_read_interrupt_interval: Duration,

    #[serde(alias = "input_raw_refresh_interval")]
    pub _input_raw_refresh_interval: u16,
    #[serde(skip)]
    pub input_raw_refresh_interval: Duration,

    #[serde(alias = "input_buffer_refresh_interval")]
    pub _input_buffer_refresh_interval: u16,
    #[serde(skip)]
    pub input_buffer_refresh_interval: Duration,

    #[serde(alias = "mouse_refresh_interval")]
    pub _mouse_refresh_interval: u16,
    #[serde(skip)]
    pub mouse_refresh_interval: Duration,
}

impl MainGeneralConfigs {
    pub fn load(&mut self) {
        self.steamy_read_interrupt_interval = Duration::from_millis(
            self._steamy_read_interrupt_interval as u64
        );
        self.input_raw_refresh_interval = Duration::from_micros(
            self._input_raw_refresh_interval as u64
        );
        self.input_buffer_refresh_interval = Duration::from_millis(
            self._input_buffer_refresh_interval as u64
        );
        self.mouse_refresh_interval = Duration::from_millis(
            self._mouse_refresh_interval as u64
        );
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LayoutNamesConfigs {
    #[serde(alias = "typing_layout")]
    pub typing_layout_name: String,
    #[serde(alias = "buttons_layout")]
    pub buttons_layout_name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MainConfigs {
    pub is_left_pad_bytes_dump: bool,
    #[serde(alias = "Layouts")]
    pub layout_names_cfg: LayoutNamesConfigs,
    #[serde(alias = "General")]
    pub general: MainGeneralConfigs,

    #[serde(skip)]
    pub layout_configs: LayoutConfigs,
}

pub fn convert_pct(value: u8) -> f32 {
    value as f32 / 100f32
}

impl MainConfigs {
    pub fn load() -> Result<Self> {
        let configs_dir = get_project_dir(PROJECT_NAME).unwrap().join("config");
        let layouts_dir = configs_dir.join("layouts");

        let mut main_configs: Self = read_toml(configs_dir.as_path(), "configs")?;

        main_configs.general.load();

        main_configs.layout_configs =
            LayoutConfigs::load(main_configs.layout_names_cfg.buttons_layout_name.as_str(), layouts_dir.as_path())?;

        Ok(main_configs)
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ZoneMappingConfigs {
    pub diagonal_zones: bool,
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
pub struct LayoutGeneralConfigs {
    pub gaming_mode: bool,
    pub repeat_keys: bool,
    pub steamy_use_left_pad: bool,
    #[serde(alias = "triggers_threshold_pct")]
    _triggers_threshold_pct: u8,
    #[serde(skip)]
    pub triggers_threshold: f32,
    pub mouse_speed: u16,
}

impl LayoutGeneralConfigs {
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
    pub general: LayoutGeneralConfigs,

    #[serde(alias = "FingerRotation")]
    pub _finger_rotation_cfg: Option<FingerRotationConfigs>,
    #[serde(skip)]
    pub finger_rotation_cfg: FingerRotationConfigs,

    #[serde(alias = "AxisCorrection")]
    pub axis_correction_cfg: AxisCorrectionConfigs,

    #[serde(alias = "Stick")]
    pub stick_zones_cfg: ZoneMappingConfigs,

    #[serde(alias = "WASD")]
    _wasd: Option<ZoneMappingConfigs>,
    #[serde(skip)]
    pub wasd_zones_cfg: ZoneMappingConfigs,

    #[serde(alias = "Scroll")]
    _scroll: Option<ScrollConfigs>,
    #[serde(skip)]
    pub scroll_cfg: ScrollConfigs,

    #[serde(alias = "JitterThreshold")]
    pub jitter_threshold_cfg: JitterThresholdConfigs,
}

impl LayoutConfigs {
    pub fn load<S: AsRef<str>, P: AsRef<Path>>(layout_name: S, layout_dir: P) -> Result<Self> {
        let mut layout_configs: Self = read_toml(layout_dir.as_ref(), layout_name)?;

        layout_configs.general.load()?;
        let gaming_mode = layout_configs.general.gaming_mode;

        match gaming_mode {
            true => {
                match layout_configs._wasd {
                    None => {
                        bail!("[WASD] has to be specified in gaming mode")
                    }
                    Some(ref wasd) => {
                        layout_configs.wasd_zones_cfg = wasd.load_and_return()?;
                    }
                }
            }
            false => match layout_configs._scroll {
                None => {
                    bail!("[Scroll] has to be specified in desktop mode")
                }
                Some(scroll) => {
                    layout_configs.scroll_cfg = scroll;
                }
            },
        }

        layout_configs.stick_zones_cfg.load()?;
        layout_configs.axis_correction_cfg.load();
        layout_configs.finger_rotation_cfg = layout_configs._finger_rotation_cfg.unwrap_or_else(|| FingerRotationConfigs {
            use_rotation: false,
            left_pad: 0,
            right_pad: 0,
            stick: 0,
        });

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
