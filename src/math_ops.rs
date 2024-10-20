use crate::pads_ops::CoordState::Value;
use crate::pads_ops::{CoordState, Coords};
use crate::utils::{are_options_different, option_to_string, Container, ContainerElement};
use color_eyre::eyre::{bail, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::ops::Add;
use strum_macros::{AsRefStr, Display, EnumIter, EnumString};
use trait_set::trait_set;

fn smoothing_factor(t_e: f64, cutoff: f64) -> f64 {
    let r = 2.0 * std::f64::consts::PI * cutoff * t_e;
    r / (r + 1.0)
}

fn exponential_smoothing(a: f64, x: f64, x_prev: f64) -> f64 {
    a * x + (1.0 - a) * x_prev
}

fn create_filter(cutoff: f64, beta: f64) -> impl FnMut(f64, f64) -> f64 {
    let mut self_filter = Filter {
        cutoff,
        beta,
        d_cutoff: 1.0,
        dx0: 0.0,
        x_prev: 0.0,
        dx_prev: 0.0,
        t_prev: 0.0,
    };

    move |t: f64, x: f64| {
        let t_e = t - self_filter.t_prev;
        let a_d = smoothing_factor(t_e, self_filter.d_cutoff);
        let dx = (x - self_filter.x_prev) / t_e;
        let dx_hat = exponential_smoothing(a_d, dx, self_filter.dx_prev);
        let cutoff = self_filter.cutoff + self_filter.beta * dx_hat.abs();
        let a = smoothing_factor(t_e, cutoff);
        let x_hat = exponential_smoothing(a, x, self_filter.x_prev);
        self_filter.x_prev = x_hat;
        self_filter.dx_prev = dx_hat;
        self_filter.t_prev = t;
        x_hat
    }
}

struct Filter {
    cutoff: f64,
    beta: f64,
    d_cutoff: f64,
    dx0: f64,
    x_prev: f64,
    dx_prev: f64,
    t_prev: f64,
}

// fn main() {
//     let filter = create_filter(1.0, 0.0);
//     let result = filter(0.0, 0.0);
//     println!("{}", result);
// }

trait_set! {
    pub trait Numeric<T> = Copy +
    Sized +
    core::convert::Into<f64> +
    core::ops::Mul<T, Output=T> +
    core::ops::MulAssign<T> +
    core::ops::Div<T, Output=T> +
    core::ops::DivAssign<T> +
    core::ops::Add<T, Output=T> +
    core::ops::AddAssign<T> +
    core::ops::Sub<T, Output=T> +
    core::ops::SubAssign<T> +
    core::ops::Rem<T, Output=T> +
    core::ops::RemAssign<T> +
    core::ops::Neg<Output=T>;
}

// pub fn hypot<T: Numeric<T>>(a: T, b: T) -> f64 {
//     (a * a + b * b).into().sqrt()
// }

const RADIANS_TO_DEGREES: f32 = 180f32 / std::f32::consts::PI;
const DEGREES_TO_RADIANS: f32 = std::f32::consts::PI / 180f32;

#[inline]
pub fn resolve_angle(angle: f32) -> f32 {
    // Important to round the source angle in the beginning, not the result.
    // Otherwise, value can become greater than 360 if the source angle is greater than 359.5 but lower than 360
    (angle.round() + 360.0) % 360.0
}

// pub fn atan2(x: f32, y: f32) -> f32 {
//     if x > 0.0 {
//         (y / x).atan()
//     } else if x < 0.0 && y >= 0.0 {
//         (y / x).atan() + std::f32::consts::PI
//     } else if x < 0.0 && y < 0.0 {
//         (y / x).atan() - std::f32::consts::PI
//     } else if x == 0.0 && y > 0.0 {
//         std::f32::consts::PI / 2.0
//     } else if x == 0.0 && y < 0.0 {
//         -(std::f32::consts::PI / 2.0)
//     } else if x == 0.0 && y == 0.0 {
//         0.0  //represents undefined
//     } else {
//         10000.0
//     }
// }

#[inline]
pub fn calc_angle(x: f32, y: f32) -> f32 {
    let angle_in_radians = y.atan2(x);
    // let angle_in_degrees = angle_in_radians.to_degrees();
    let angle_in_degrees = angle_in_radians * RADIANS_TO_DEGREES;
    let angle = resolve_angle(angle_in_degrees);

    if angle < 0.0 || angle >= 360.0 {
        panic!("Incorrect angle: '{}'", angle)
    }
    angle
}

#[inline]
pub fn distance(x: f32, y: f32) -> f32 {
    x.hypot(y)
}

#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    #[inline(always)]
    pub fn as_coords(&self) -> Coords {
        Coords {
            x: Value(self.x),
            y: Value(self.y),
        }
    }

    #[inline(always)]
    pub fn from_2_coords(point1: Coords, point2: Coords) -> Option<Vector> {
        match (point1.x, point1.y, point2.x, point2.y) {
            (Value(point1_x), Value(point1_y), Value(point2_x), Value(point2_y)) => Some(Self {
                x: point2_x - point1_x,
                y: point2_y - point1_y,
            }),
            _ => None,
        }
    }

    //color-eyre takes a lot of CPU when frequently constructing error messages
    #[inline(always)]
    pub fn from_coords(coords: Coords) -> Option<Self> {
        match (coords.x, coords.y) {
            (Value(x), Value(y)) => Some(Self { x, y }),
            _ => None,
        }
    }

    #[inline]
    pub fn angle(&self) -> f32 {
        calc_angle(self.x, self.y)
    }

    #[inline]
    pub fn distance(&self) -> f32 {
        self.x.hypot(self.y)
    }

    #[inline]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;

    #[inline]
    fn add(self, other: Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::AddAssign<Vector> for Vector {
    #[inline]
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Vector;

    #[inline]
    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::SubAssign<Vector> for Vector {
    #[inline]
    fn sub_assign(&mut self, other: Vector) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

#[inline]
pub fn rotate_by_angle(point1: Vector, mut point2: Vector, rotation_angle: f32) -> Vector {
    let rotation_angle = rotation_angle * DEGREES_TO_RADIANS;
    let sin: f32 = rotation_angle.sin();
    let cos: f32 = rotation_angle.cos();

    point2 -= point1;

    let mut rotated_point = Vector {
        x: point2.x * cos - point2.y * sin,
        y: point2.x * sin + point2.y * cos,
    };

    rotated_point += point1;

    rotated_point
}

#[derive(
    PartialOrd,
    EnumIter,
    EnumString,
    AsRefStr,
    Display,
    Eq,
    Hash,
    PartialEq,
    Copy,
    Clone,
    Debug,
    Serialize,
    Deserialize,
)]
pub enum Quadrant {
    Q1,
    Q2,
    Q3,
    Q4,
}

#[inline]
fn get_quadrant(angle: Angle) -> Quadrant {
    let angle = resolve_angle(angle as f32) as Angle;
    if angle >= 270 {
        Quadrant::Q4
    } else if angle >= 180 {
        Quadrant::Q3
    } else if angle >= 90 {
        Quadrant::Q2
    } else {
        Quadrant::Q1
    }
}

#[inline]
fn get_rotation_by_quadrant(source_angle: Angle, rotation: Angle) -> Angle {
    match get_quadrant(source_angle) {
        Quadrant::Q1 => rotation,
        Quadrant::Q2 => rotation,
        Quadrant::Q3 => rotation / 2,
        Quadrant::Q4 => rotation / 2,
    }
}

#[inline]
pub fn rotate_around_center(point: Vector, rotation_angle: f32) -> Vector {
    rotate_by_angle(Vector::zero(), point, rotation_angle)
}

pub fn convert_range<T: Numeric<T>>(
    input: T,
    input_start: T,
    input_end: T,
    output_start: T,
    output_end: T,
) -> T {
    /* Note, "slope" below is a constant for given numbers, so if you are calculating
    a lot of output values, it makes sense to calculate it once.  It also makes
    understanding the code easier */
    let slope = (output_end - output_start) / (input_end - input_start);
    let output = output_start + slope * (input - input_start);
    output
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RangeConverterBuilder<T: Numeric<T>> {
    slope: T,
    pre_calc: T,
}

impl<T: Numeric<T>> RangeConverterBuilder<T> {
    pub fn build(input_start: T, input_end: T, output_start: T, output_end: T) -> Self {
        let slope = (output_end - output_start) / (input_end - input_start);

        // let output = output_start + slope * (input - input_start);
        // let output = output_start + slope * input - slope * input_start;
        // let output = slope * input + output_start - slope * input_start;
        let pre_calc = output_start - slope * input_start;
        // let output = slope * input + pre_calc;

        Self { slope, pre_calc }
    }

    #[inline]
    pub fn convert(&self, input: T) -> T {
        let output = self.slope * input + self.pre_calc;
        output
    }
}

pub type Angle = usize;
pub type ZoneNumber = u8;

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ZoneAllowedRange {
    vertical: Angle,
    horizontal: Angle,
    diagonal: Angle,
}

impl ZoneAllowedRange {
    pub fn new(vertical: Angle, horizontal: Angle, diagonal: Angle) -> Result<Self> {
        if vertical + horizontal + 2 * diagonal > 90 {
            bail!(
                "Incorrect zones allowed range. Vertical: {}, Horizontal: {}, Diagonal: {}",
                vertical,
                horizontal,
                diagonal
            )
        }
        Ok(Self {
            vertical,
            horizontal,
            diagonal,
        })
    }

    pub fn from_one_value(range: Angle, diagonal_zones: bool) -> Result<Self> {
        match diagonal_zones {
            true => Self::new(range, range, range),
            false => Self::new(range, range, 0),
        }
    }
}

pub fn pivot_angle_to_allowed_range(
    angle: Angle,
    zone_allowed_range: &ZoneAllowedRange,
) -> Result<Angle> {
    if angle % 180 == 0 {
        return Ok(zone_allowed_range.horizontal);
    } else if angle % 90 == 0 {
        return Ok(zone_allowed_range.vertical);
    } else if angle % 45 == 0 {
        return Ok(zone_allowed_range.diagonal);
    } else {
        bail!("Such angle cannot be converted: '{}'", angle)
    }
}
trait_set! {
    pub trait ZoneValue = ContainerElement;
}

#[derive(PartialEq, Clone, Debug)]
pub struct ZonesMapper<T: ZoneValue> {
    angle_to_value: [Option<Vec<T>>; 360],
    angle_to_zone: [Option<ZoneNumber>; 360],
    prev_zone: Option<ZoneNumber>,
    prev_value: Option<Vec<T>>,
    threshold: f32,
}

impl<T: ZoneValue> ZonesMapper<T> {
    #[inline]
    pub fn get_commands_diff(&mut self, x: CoordState, y: CoordState) -> (Vec<T>, Vec<T>, Vec<T>) {
        let (zone_changed, cur_value) = self.detect_zone(x, y);
        let prev_value = self.prev_value.clone();
        self.prev_value = cur_value.clone();

        match zone_changed {
            true => match (prev_value, cur_value) {
                (Some(prev_value), Some(cur_value)) => {
                    let to_press_full = cur_value.clone();

                    let prev_value = Container::from(prev_value);
                    let cur_value = Container::from(cur_value);

                    let to_release = prev_value.difference(&cur_value);
                    let to_press = cur_value.difference(&prev_value);

                    // let mut to_release = vec![];
                    // for prev_element_val in &prev_value {
                    //     if !cur_value.contains(prev_element_val) {
                    //         to_release.push(prev_element_val.clone());
                    //     }
                    // }
                    // let mut to_press = vec![];
                    // for cur_element_val in &cur_value {
                    //     if !prev_value.contains(cur_element_val) {
                    //         to_press.push(cur_element_val.clone());
                    //     }
                    // }

                    (to_release, to_press, to_press_full)
                }
                (Some(prev_value), None) => (prev_value.clone(), vec![], vec![]),
                (None, Some(cur_value)) => (vec![], cur_value.clone(), cur_value),
                (None, None) => (vec![], vec![], vec![]),
            },
            false => (vec![], vec![], vec![]),
        }
    }

    #[inline]
    pub fn detect_zone(&mut self, x: CoordState, y: CoordState) -> (bool, Option<Vec<T>>) {
        let (zone, angle) = match (x, y) {
            (Value(x), Value(y)) => {
                if distance(x, y) > self.threshold {
                    // debug!("Angle: {}", calc_angle(x, y));
                    let angle = calc_angle(x, y) as Angle;
                    (self.angle_to_zone[angle], Some(angle))
                } else {
                    (None, None)
                }
            }
            _ => (None, None),
        };
        // debug!("Prev zone: '{:?}'; Cur zone: '{:?}'", self.prev_zone, zone);

        let zone_changed = are_options_different(self.prev_zone, zone);
        // debug!("Changed: {}", zone_changed);
        self.prev_zone = zone;
        let value = angle.and_then(|angle| self.angle_to_value[angle].clone());
        (zone_changed, value)
    }

    pub fn gen_from(
        values: Vec<Vec<T>>,
        start_angle: Angle,
        zone_allowed_range: &ZoneAllowedRange,
        threshold: f32,
        diagonal_zones: bool,
    ) -> Result<Self> {
        match diagonal_zones {
            true => {
                if values.len() != 4 {
                    bail!(
                        "4 values have to be provided to build diagonal zone mapper. Provided: {}",
                        values.len()
                    )
                }
                let values: [Vec<T>; 4] = [
                    values[0].clone(),
                    values[1].clone(),
                    values[2].clone(),
                    values[3].clone(),
                ];
                Self::_gen_from_4_into_8(values, start_angle, zone_allowed_range, threshold)
            }
            false => Self::_gen_from_any(values, start_angle, zone_allowed_range, threshold),
        }
    }

    fn _gen_from_4_into_8(
        values: [Vec<T>; 4],
        start_angle: Angle,
        zone_allowed_range: &ZoneAllowedRange,
        threshold: f32,
    ) -> Result<Self> {
        //initialize with copy of first value because KeyCode (used as generic type T) doesn't implement Default trait
        let mut expanded_values: [Vec<T>; 8] = core::array::from_fn(|_| values[0].clone());
        for ind in 0..values.len() {
            expanded_values[ind * 2] = values[ind].clone();
            expanded_values[ind * 2 + 1] = [
                values[ind].clone(),
                values[(ind + 1) % values.len()].clone(),
            ]
                .concat();
        }

        Self::_gen_from_any(
            expanded_values.to_vec(),
            start_angle,
            zone_allowed_range,
            threshold,
        )
    }

    fn _print_angle_to_value(angle_to_value: &[Option<Vec<T>>; 360]) {
        for ind in 0..angle_to_value.len() {
            let value = match angle_to_value[ind].clone() {
                None => "None".to_string(),
                Some(value) => {
                    let mut combined_str = String::new();
                    for val in value {
                        combined_str = combined_str.add(format!("{} ", val).as_str());
                    }
                    combined_str
                }
            };
            debug!("{}: {}", ind, value)
        }
    }

    fn _print_angle_to_zone(angle_to_zone: &[Option<ZoneNumber>; 360]) {
        for ind in 0..angle_to_zone.len() {
            let value = option_to_string(angle_to_zone[ind]);
            debug!("{}: {}", ind, value);
        }
    }

    fn _gen_from_any(
        values: Vec<Vec<T>>,
        start_angle: Angle,
        zone_allowed_range: &ZoneAllowedRange,
        threshold: f32,
    ) -> Result<Self> {
        let sectors_amount = values.len();
        if sectors_amount > ZoneNumber::MAX as usize {
            bail!(
                "Maximum amount of zone values is '{}' but provided '{}'",
                ZoneNumber::MAX,
                sectors_amount
            )
        }

        let sector_size = 360.0 / sectors_amount as f32;
        if sector_size.fract() != 0.0 {
            bail!(
                "Incorrect amount of zone values to form sectors. Sector's size is not an integer"
            )
        }
        let sector_size = sector_size as Angle;

        let mut angle_to_value: [Option<Vec<T>>; 360] = std::array::from_fn(|_| None);
        let mut angle_to_zone: [Option<ZoneNumber>; 360] = std::array::from_fn(|_| None);

        for ind in 0..sectors_amount {
            let pivot_angle = start_angle + sector_size * ind;
            let allowed_range = pivot_angle_to_allowed_range(pivot_angle, zone_allowed_range)?;
            let range_to_value = Self::gen_range(pivot_angle, allowed_range, &values[ind]);
            for (angle, value) in range_to_value {
                if angle >= 360 || angle < 0 {
                    bail!("Incorrectly generated angle '{}'", angle)
                }
                if angle_to_value[angle].is_some() {
                    bail!("Duplicate angle '{}'", angle)
                }
                angle_to_value[angle] = Some(value.clone());
                angle_to_zone[angle] = Some(ind as ZoneNumber);
            }
        }

        Self::_print_angle_to_value(&angle_to_value);
        Self::_print_angle_to_zone(&angle_to_zone);

        Ok(Self {
            angle_to_value,
            angle_to_zone,
            threshold,
            prev_zone: None,
            prev_value: None,
        })
    }

    pub fn gen_range(
        pivot_angle: Angle,
        allowed_range: Angle,
        value: &Vec<T>,
    ) -> Vec<(Angle, &Vec<T>)> {
        let range_start = 360 + pivot_angle - allowed_range;
        // +1 to include end of range and close the gap between zones
        let range_end = 360 + pivot_angle + allowed_range + 1;

        let mut angle_to_value = vec![];
        for angle in range_start..range_end {
            let angle = angle % 360;
            angle_to_value.push((angle, value))
        }
        angle_to_value
    }
}

const MAX_COORD_VALUE: f32 = i16::MAX as f32;
const MIN_COORD_VALUE: f32 = i16::MIN as f32;

pub fn coord_to_f32(value: i16) -> f32 {
    let mut value = value as f32; //first convert to f32 otherwise i16 will overflow

    if value == 0.0 {
        0.0
    } else if value > 0.0 {
        value / MAX_COORD_VALUE
    } else {
        value / MIN_COORD_VALUE.abs()
    }
}

pub fn apply_pad_stick_correction(orig: f32, correction: f32) -> f32 {
    let mut value = orig + correction;
    value = value.clamp(-1.0, 1.0);
    value
}
