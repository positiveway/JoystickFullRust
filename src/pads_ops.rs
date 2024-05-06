use mouse_keyboard_input::Coord;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use crate::configs::FingerRotation;
use crate::math_ops::{rotate_around_center, Vector};
use log::{debug, info};


#[derive(Display, Eq, Hash, PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub enum MouseMode {
    #[default]
    CursorMove,
    Typing,
}

#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Coords {
    pub x: Option<f32>,
    pub y: Option<f32>,
}

impl Coords {
    #[inline]
    pub fn reset(&mut self) {
        self.x = None;
        self.y = None;
    }

    #[inline]
    pub fn update_one_coord(prev: &mut Option<f32>, new: Option<f32>) {
        if new.is_some() {
            *prev = new;
        }
    }

    #[inline]
    pub fn update_one_if_not_init(prev: &mut Option<f32>, new: Option<f32>) {
        if prev.is_none() {
            Self::update_one_coord(prev, new);
        }
    }

    #[inline]
    pub fn update(&mut self, new: &Self) {
        Self::update_one_coord(&mut self.x, new.x);
        Self::update_one_coord(&mut self.y, new.y);
    }

    #[inline]
    pub fn update_if_not_init(&mut self, new: &Self) {
        Self::update_one_if_not_init(&mut self.x, new.x);
        Self::update_one_if_not_init(&mut self.y, new.y);
    }

    //pub fn set_one_coord(cur: &mut Option<f32>, prev: Option<f32>) {
    //     if cur.is_none() {
    //         *cur = prev;
    //     }
    // }

    // pub fn set_prev_if_cur_is_none(&mut self, prev: &Self) {
    //     Self::set_one_coord(&mut self.x, prev.x);
    //     Self::set_one_coord(&mut self.y, prev.y);
    // }

    #[inline]
    pub fn any_is_not_init(&self) -> bool {
        self.x.is_none() || self.y.is_none()
    }

    #[inline]
    pub fn any_changes(&self) -> bool {
        self.x.is_some() || self.y.is_some()
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
            (Some(x), Some(y)) => x.hypot(y),
            (_, _) => 0.0,
        }
    }
}

#[inline]
pub fn option_to_string(value: Option<f32>) -> String {
    match value {
        None => "None".to_string(),
        Some(value) => value.to_string(),
    }
}

impl std::fmt::Display for Coords {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "[x: {}, y: {}]",
            option_to_string(self.x),
            option_to_string(self.y)
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
    pub x: Coord,
    pub y: Coord,
}

impl ConvertedCoordsDiff {
    #[inline]
    pub fn is_any_changes(&self) -> bool {
        self.x != 0 || self.y != 0
    }
}

#[inline]
pub fn calc_diff_one_coord(prev_coord: Option<f32>, cur_coord: Option<f32>) -> f32 {
    match (prev_coord, cur_coord) {
        (Some(prev_value), Some(cur_value)) => cur_value - prev_value,
        _ => 0.0,
    }
}

#[inline]
pub fn convert_diff(value: f32, multiplier: u16) -> Coord {
    (value * multiplier as f32).round() as Coord
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct CoordsState {
    pub prev: Coords,
    pub cur: Coords,
    pub finger_rotation: i16,
    pub use_rotation: bool,
}

impl CoordsState {
    pub fn new(finger_rotation: i16, use_rotation: bool) -> Self {
        Self {
            prev: Default::default(),
            cur: Default::default(),
            finger_rotation,
            use_rotation,
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
    pub fn reset(&mut self) {
        self.prev.reset();
        self.cur.reset();
    }

    #[inline]
    pub fn reset_current(&mut self) {
        self.cur.reset();
    }

    #[inline]
    pub fn update(&mut self) {
        self.prev.update(&self.cur)
    }

    #[inline]
    pub fn update_if_not_init(&mut self) {
        self.prev.update_if_not_init(&self.cur);
    }

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
        let (prev_coords, cur_coords) = match self.use_rotation {
            true => (
                self.prev.try_rotate(self.finger_rotation),
                self.cur_pos().try_rotate(self.finger_rotation),
            ),
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
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PadsCoords {
    pub left_pad: CoordsState,
    pub right_pad: CoordsState,
    pub stick: CoordsState,
}

impl PadsCoords {
    pub fn new(finger_rotation: &FingerRotation) -> Self {
        Self {
            left_pad: CoordsState::new(finger_rotation.left_pad, finger_rotation.use_rotation),
            right_pad: CoordsState::new(finger_rotation.right_pad, finger_rotation.use_rotation),
            stick: CoordsState::new(finger_rotation.stick, finger_rotation.use_rotation),
        }
    }

    #[inline]
    pub fn reset(&mut self) {
        self.left_pad.reset();
        self.right_pad.reset();
        self.stick.reset();
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

    #[inline]
    pub fn update_if_not_init(&mut self) {
        self.left_pad.update_if_not_init();
        self.right_pad.update_if_not_init();
        self.stick.update_if_not_init();
    }
}

#[inline(always)]
pub fn is_jitter(value: f32, jitter_threshold: f32) -> bool {
    value.abs() <= jitter_threshold
}

#[inline]
pub fn discard_jitter(prev_value: Option<f32>, new_value: f32, jitter_threshold: f32) -> Option<f32> {
    match prev_value {
        None => Some(new_value),
        Some(prev_value) => {
            let diff = new_value - prev_value;
            match is_jitter(diff, jitter_threshold) {
                true => None,
                false => Some(new_value),
            }
        }
    }
}