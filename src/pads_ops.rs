use std::fmt::{Display, Formatter};
use crate::configs::{AxisCorrection, AxisCorrectionConfigs, FingerRotationConfigs, JitterThresholdConfigs, ZoneMappingConfigs};
use crate::math_ops::{rotate_around_center, Vector, ZonesMapper, ZoneValue};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use universal_input::{OS_Input_Coord, KeyCode};
use universal_input::KeyCode::KEY_LEFTSHIFT;
use crate::buttons_state::ButtonsState;
use crate::pads_ops::CoordState::Value;
use crate::steamy_state::SteamyInputCoord;
use crate::utils::{are_options_different, option_to_string};

#[derive(PartialOrd, EnumIter, EnumString, AsRefStr, Display, Default, Eq, Hash, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, )]
pub enum MouseMode {
    #[default]
    CursorMove,
    Typing,
}

#[derive(PartialOrd, EnumIter, EnumString, AsRefStr, Default, PartialEq, Copy, Clone, Debug, Serialize, Deserialize, )]
pub enum CoordState {
    #[default]
    NotInit,
    DiscardNext,
    Value(f32),
}

impl Display for CoordState {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Self::NotInit => write!(f, "NotInit"),
            Self::DiscardNext => write!(f, "DiscardNext"),
            Self::Value(value) => write!(f, "{}", value),
        }
    }
}

impl CoordState {
    pub fn has_value(&self) -> bool {
        match self {
            CoordState::Value(_) => true,
            _ => false,
        }
    }

    pub fn not_init(&self) -> bool {
        match self {
            CoordState::NotInit => true,
            _ => false,
        }
    }

    pub fn or(self, other: Self) -> Self {
        match self {
            value @ Value(_) => value,
            CoordState::NotInit => other,
            CoordState::DiscardNext => CoordState::DiscardNext //TODO: Check this invariant
        }
    }
}

#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Coords {
    pub x: CoordState,
    pub y: CoordState,
}

impl Coords {
    #[inline]
    pub fn reset(&mut self) {
        self.x = CoordState::NotInit;
        self.y = CoordState::NotInit;
    }

    #[inline]
    pub fn set_to_discard_next(&mut self) {
        self.x = CoordState::DiscardNext;
        self.y = CoordState::DiscardNext;
    }

    #[inline]
    pub fn update_one_coord(prev: &mut CoordState, new: CoordState) {
        if new.has_value() {
            *prev = new;
        }
    }

    // #[inline]
    // pub fn update_one_if_not_init(prev: &mut Option<f32>, new: Option<f32>) {
    //     if prev.is_none() {
    //         Self::update_one_coord(prev, new);
    //     }
    // }

    #[inline]
    pub fn update(&mut self, new: &Self) {
        Self::update_one_coord(&mut self.x, new.x);
        Self::update_one_coord(&mut self.y, new.y);
    }

    // #[inline]
    // pub fn update_if_not_init(&mut self, new: &Self) {
    //     Self::update_one_if_not_init(&mut self.x, new.x);
    //     Self::update_one_if_not_init(&mut self.y, new.y);
    // }

    //pub fn set_one_coord(cur: &mut Option<f32>, prev: Option<f32>) {
    //     if cur.is_none() {
    //         *cur = prev;
    //     }
    // }

    // pub fn set_prev_if_cur_is_none(&mut self, prev: &Self) {
    //     Self::set_one_coord(&mut self.x, prev.x);
    //     Self::set_one_coord(&mut self.y, prev.y);
    // }

    // #[inline]
    // pub fn any_is_not_init(&self) -> bool {
    //     self.x.is_none() || self.y.is_none()
    // }

    #[inline]
    pub fn any_changes(&self) -> bool {
        self.x.has_value() || self.y.has_value()
    }

    #[inline]
    pub fn rotate(&self, rotation: i16) -> Option<Self> {
        Vector::from_coords(*self).and_then(|vector: Vector| {
            Some(rotate_around_center(vector, rotation as f32).as_coords())
        })
    }

    #[inline]
    pub fn try_rotate(&self, rotation: i16) -> Self {
        self.rotate(rotation).unwrap_or(*self)
    }

    #[inline]
    pub fn magnitude(&self) -> f32 {
        match (self.x, self.y) {
            (Value(x), Value(y)) => x.hypot(y),
            (_, _) => 0.0,
        }
    }
}

impl std::fmt::Display for Coords {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "[X: {}, Y: {}]",
            self.x,
            self.x
        )
    }
}

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CoordsDiff {
    pub x: f32,
    pub y: f32,
}

impl CoordsDiff {
    #[inline]
    pub fn convert(&self, multiplier: u16) -> ConvertedCoordsDiff {
        ConvertedCoordsDiff {
            x: convert_diff(self.x, multiplier),
            y: convert_diff(self.y, multiplier),
        }
    }

    #[inline]
    pub fn is_any_changes(&self) -> bool {
        self.x != 0.0 || self.y != 0.0
    }
}

#[derive(PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ConvertedCoordsDiff {
    pub x: OS_Input_Coord,
    pub y: OS_Input_Coord,
}

impl ConvertedCoordsDiff {
    #[inline]
    pub fn is_any_changes(&self) -> bool {
        self.x != 0 || self.y != 0
    }
}

#[inline]
pub fn calc_diff_one_coord(prev_coord: CoordState, cur_coord: CoordState) -> f32 {
    match (prev_coord, cur_coord) {
        (Value(prev_value), Value(cur_value)) => cur_value - prev_value,
        _ => 0.0,
    }
}

#[inline]
pub fn convert_diff(value: f32, multiplier: u16) -> OS_Input_Coord {
    (value * multiplier as f32).round() as OS_Input_Coord
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CoordsHistoryState {
    pub prev: Coords,
    pub cur: Coords,
    pub finger_rotation: i16,
    pub use_rotation: bool,
    pub axis_correction: AxisCorrection,
    pub use_correction: bool,
    pub jitter_threshold: f32,
}

impl CoordsHistoryState {
    pub fn new(
        finger_rotation: i16,
        use_rotation: bool,
        axis_correction: AxisCorrection,
        use_correction: bool,
        jitter_threshold: f32,
    ) -> Self {
        Self {
            prev: Default::default(),
            cur: Default::default(),
            finger_rotation,
            use_rotation,
            axis_correction,
            use_correction,
            jitter_threshold,
        }
    }

    #[inline]
    pub fn any_changes(&self) -> bool {
        self.cur.any_changes()
    }

    // pub fn set_prev_if_cur_is_none(&mut self) {
    //     self.cur.set_prev_if_cur_is_none(&self.prev);
    // }

    #[inline]
    pub fn reset_all(&mut self) {
        self.prev.reset();
        self.cur.reset();
    }

    #[inline]
    pub fn set_to_discard_next(&mut self) {
        self.prev.set_to_discard_next();
    }

    #[inline]
    pub fn reset_current(&mut self) {
        self.cur.reset();
    }

    #[inline]
    pub fn update(&mut self) {
        self.prev.update(&self.cur)
    }

    // #[inline]
    // pub fn update_if_not_init(&mut self) {
    //     self.prev.update_if_not_init(&self.cur);
    // }

    //Left for DEBUG reference
    // pub fn rotate_cur_coords(&self) -> Option<Coords> {
    // let cur_pos = self.cur_pos();
    // match (
    //     cur_pos.x,
    //     cur_pos.y,
    // ) {
    //     (Some(x), Some(y)) => {
    //         let point = Vector { x, y };
    //
    //         let orig_angle = point.angle();
    //         let rotated_vector = rotate_around_center(point, self.finger_rotation as f32);
    //
    //         let rotated_coords = rotated_vector.as_coords();
    //         debug!("Origin: {}", self.cur);
    //         debug!("Filled: {}", self.cur_pos());
    //         debug!("Rotated: {}", rotated_coords);
    //         debug!(
    //             "Angle: [Orig: {}, Shifted: {}; Rotation: {}]",
    //             orig_angle,
    //             rotated_vector.angle(),
    //             self.finger_rotation
    //         );
    //         debug!("");
    //         Some(rotated_coords)
    //     }
    //     _ => None,
    // }
    // }

    #[inline]
    pub fn cur_pos(&self) -> Coords {
        Coords {
            x: self.cur.x.or(self.prev.x),
            y: self.cur.y.or(self.prev.y),
        }
    }

    #[inline]
    pub fn diff(&mut self) -> CoordsDiff {
        debug!(
            "Prev: [X: {}, Y: {}]. Cur: [X: {}, Y: {}]",
            self.prev.x,
            self.prev.y,
            self.cur.x,
            self.cur.y,
        );
        let (prev_coords, cur_coords) = match self.use_rotation {
            true => (
                self.prev.try_rotate(self.finger_rotation),
                //Check it for default for proper diff calculations
                // self.cur_pos().rotate(self.finger_rotation).unwrap_or(self.cur),
                self.cur_pos().try_rotate(self.finger_rotation),
            ),
            // false => (self.prev, self.cur),
            false => (self.prev, self.cur_pos()),
        };

        let diff = CoordsDiff {
            x: calc_diff_one_coord(prev_coords.x, cur_coords.x),
            y: calc_diff_one_coord(prev_coords.y, cur_coords.y),
        };
        diff
    }

    #[inline]
    pub fn convert_diff(&mut self, multiplier: u16) -> ConvertedCoordsDiff {
        self.diff().convert(multiplier)
    }

    // pub fn diff_and_update(&mut self) -> CoordsDiff {
    //     let diff = self.diff();
    //     if diff.is_any_changes() {
    //         self.update();
    //     }
    //     diff
    // }
    //
    // pub fn convert_diff_and_update(&mut self, multiplier: u16) -> ConvertedCoordsDiff {
    //     let converted_diff = self.convert_diff(multiplier);
    //     if converted_diff.is_any_changes() {
    //         self.update();
    //     }
    //     converted_diff
    // }

    #[inline]
    pub fn send_commands_diff(
        &self,
        zone_mapper: &mut ZonesMapper<KeyCode>,
        mapping_configs: &ZoneMappingConfigs,
        buttons_state: &mut ButtonsState,
        always_press: bool,
    ) -> color_eyre::Result<()> {
        let cur_pos = self.cur_pos().try_rotate(self.finger_rotation);

        let (to_release, to_press, to_press_full) =
            zone_mapper.get_commands_diff(cur_pos.x, cur_pos.y);
        // if !to_release.is_empty() || !to_press.is_empty() {
        //     println!("To release: '{:?}'; To press: '{:?}'", to_release, to_press)
        // }

        let to_press = if always_press {
            to_press_full
        } else {
            to_press
        };

        //Press goes first to check if already pressed
        for keycode in to_press {
            buttons_state.press_keycodes(vec![keycode], always_press)?;
        }
        for keycode in to_release {
            buttons_state.release_keycodes(vec![keycode], false)?;
        }

        if mapping_configs.use_shift {
            if cur_pos.magnitude() > mapping_configs.shift_threshold {
                buttons_state.press_keycodes(vec![KEY_LEFTSHIFT], always_press)?;
            } else {
                buttons_state.release_keycodes(vec![KEY_LEFTSHIFT], false)?;
            }
        }

        Ok(())
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PadsCoords {
    pub left_pad: CoordsHistoryState,
    pub right_pad: CoordsHistoryState,
    pub stick: CoordsHistoryState,
}

impl PadsCoords {
    pub fn new(
        finger_rotation_cfg: &FingerRotationConfigs,
        axis_correction_cfg: &AxisCorrectionConfigs,
        jitter_threshold_cfg: &JitterThresholdConfigs,
    ) -> Self {
        let use_rotation = finger_rotation_cfg.use_rotation;
        let use_correction = axis_correction_cfg.use_correction;

        Self {
            left_pad: CoordsHistoryState::new(
                finger_rotation_cfg.left_pad,
                use_rotation,
                axis_correction_cfg.left_pad,
                use_correction,
                jitter_threshold_cfg.left_pad,
            ),
            right_pad: CoordsHistoryState::new(
                finger_rotation_cfg.right_pad,
                use_rotation,
                axis_correction_cfg.right_pad,
                use_correction,
                jitter_threshold_cfg.right_pad,
            ),
            stick: CoordsHistoryState::new(
                finger_rotation_cfg.stick,
                use_rotation,
                axis_correction_cfg.stick,
                use_correction,
                jitter_threshold_cfg.stick,
            ),
        }
    }

    #[inline]
    pub fn reset_all(&mut self) {
        self.left_pad.reset_all();
        self.right_pad.reset_all();
        self.stick.reset_all();
    }

    #[inline]
    pub fn reset_current(&mut self) {
        self.left_pad.reset_current();
        self.right_pad.reset_current();
        self.stick.reset_current();
    }

    // pub fn set_prev_if_cur_is_none(&mut self) {
    //     self.left_pad.set_prev_if_cur_is_none();
    //     self.right_pad.set_prev_if_cur_is_none();
    //     self.stick.set_prev_if_cur_is_none();
    // }

    #[inline]
    pub fn update(&mut self) {
        self.left_pad.update();
        self.right_pad.update();
        self.stick.update();
    }

    // #[inline]
    // pub fn update_if_not_init(&mut self) {
    //     self.left_pad.update_if_not_init();
    //     self.right_pad.update_if_not_init();
    //     self.stick.update_if_not_init();
    // }
}

#[inline(always)]
pub fn is_jitter(value: f32, jitter_threshold: f32) -> bool {
    value.abs() <= jitter_threshold
}

#[inline]
pub fn discard_jitter_for_pad(
    prev_value: CoordState,
    new_value: f32,
    jitter_threshold: f32,
) -> CoordState {
    match prev_value {
        CoordState::NotInit => Value(new_value),
        CoordState::DiscardNext => CoordState::NotInit,
        Value(prev_value) => {
            let diff = new_value - prev_value;
            match is_jitter(diff, jitter_threshold) {
                true => CoordState::NotInit,
                false => Value(new_value),
            }
        }
    }
}

#[inline]
pub fn discard_jitter_for_stick(
    prev_value: CoordState,
    new_value: f32,
    jitter_threshold: f32,
    correction: SteamyInputCoord,
    use_correction: bool,
) -> CoordState {
    let zero_value: f32 = match use_correction {
        true => correction as f32,
        false => 0.0
    };
    if prev_value != Value(new_value) && new_value == zero_value {
        Value(zero_value)
    } else {
        discard_jitter_for_pad(prev_value, new_value, jitter_threshold)
    }
}