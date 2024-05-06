use crate::mouse::Coords;
use color_eyre::eyre::{bail, Result};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fmt::{Display};
use std::ops::Add;
use strum_macros::Display;
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
            x: Some(self.x),
            y: Some(self.y),
        }
    }

    #[inline(always)]
    pub fn from_2_coords(point1: Coords, point2: Coords) -> Option<Vector> {
        match (point1.x, point1.y, point2.x, point2.y) {
            (Some(point1_x), Some(point1_y), Some(point2_x), Some(point2_y)) => Some(Self {
                x: point2_x - point1_x,
                y: point2_y - point1_y,
            }),
            _ => None
        }
    }

    //color-eyre takes a lot of CPU when frequently constructing error messages
    #[inline(always)]
    pub fn from_coords(coords: Coords) -> Option<Self> {
        match (coords.x, coords.y) {
            (Some(x), Some(y)) => Some(Self { x, y }),
            _ => None
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

#[derive(PartialEq, Default, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct ZoneAllowedRange {
    vertical: u16,
    horizontal: u16,
    diagonal: u16,
}

impl ZoneAllowedRange {
    pub fn new(vertical: u16, horizontal: u16, diagonal: u16) -> Result<Self> {
        if vertical + horizontal + 2 * diagonal > 90 {
            bail!("Incorrect zones allowed range")
        }
        Ok(Self {
            vertical,
            horizontal,
            diagonal,
        })
    }
}

pub fn pivot_angle_to_allowed_range(
    angle: u16,
    zone_allowed_range: &ZoneAllowedRange,
) -> Result<u16> {
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

#[inline(always)]
pub fn are_options_equal<T: PartialEq>(value1: Option<T>, value2: Option<T>) -> bool {
    match (value1, value2) {
        (Some(value1), Some(value2)) => value1 == value2,
        (None, None) => true,
        _ => false,
    }
}

#[inline(always)]
pub fn are_options_different<T: PartialEq>(value1: Option<T>, value2: Option<T>) -> bool {
    !are_options_equal(value1, value2)
}

#[derive(PartialEq, Clone, Debug)]
pub struct ZonesMapper<T: Clone + Display + PartialEq> {
    angle_to_value: [Option<Vec<T>>; 360],
    angle_to_zone: [u8; 360],
    prev_zone: Option<u8>,
    prev_value: Option<Vec<T>>,
    threshold: f32,
}

impl<T: Clone + Display + PartialEq> ZonesMapper<T> {
    #[inline]
    pub fn get_commands_diff(
        &mut self,
        x: Option<f32>,
        y: Option<f32>,
    ) -> (Vec<T>, Vec<T>, Vec<T>) {
        let (zone_changed, cur_value) = self.detect_zone(x, y);
        let prev_value = self.prev_value.clone();
        self.prev_value = cur_value.clone();

        match zone_changed {
            true => match (prev_value, cur_value) {
                (Some(prev_value), Some(cur_value)) => {
                    let mut to_release = vec![];
                    for prev_element_val in &prev_value {
                        if !cur_value.contains(prev_element_val) {
                            to_release.push(prev_element_val.clone());
                        }
                    }
                    let mut to_press = vec![];
                    for cur_element_val in &cur_value {
                        if !prev_value.contains(cur_element_val) {
                            to_press.push(cur_element_val.clone());
                        }
                    }
                    let to_press_full = cur_value.clone();
                    (to_release, to_press, to_press_full)
                }
                (Some(prev_value), None) => (prev_value.clone(), vec![], vec![]),
                (None, Some(cur_value)) => (vec![], cur_value.clone(), cur_value.clone()),
                (None, None) => (vec![], vec![], vec![]),
            },
            false => (vec![], vec![], vec![]),
        }
    }

    #[inline]
    pub fn detect_zone(&mut self, x: Option<f32>, y: Option<f32>) -> (bool, Option<Vec<T>>) {
        let (zone, angle) = match (x, y) {
            (Some(x), Some(y)) => {
                if distance(x, y) > self.threshold {
                    // debug!("Angle: {}", calc_angle(x, y));
                    let angle = calc_angle(x, y) as usize;
                    (Some(self.angle_to_zone[angle]), Some(angle))
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

    pub fn gen_from_4_into_8(
        values: [Vec<T>; 4],
        start_angle: u16,
        zone_allowed_range: &ZoneAllowedRange,
        threshold: f32,
    ) -> Result<Self> {
        let mut expanded_values: [Vec<T>; 8] = core::array::from_fn(|i| values[0].clone());
        for ind in 0..values.len() {
            expanded_values[ind * 2] = values[ind].clone();
            expanded_values[ind * 2 + 1] = [
                values[ind].clone(),
                values[(ind + 1) % values.len()].clone(),
            ]
                .concat();
        }

        Self::gen_from_8(expanded_values, start_angle, zone_allowed_range, threshold)
    }

    pub fn gen_from_8(
        values: [Vec<T>; 8],
        start_angle: u16,
        zone_allowed_range: &ZoneAllowedRange,
        threshold: f32,
    ) -> Result<Self> {
        let mut angle_to_value: [Option<Vec<T>>; 360] = std::array::from_fn(|_| None);
        let mut angle_to_zone: [u8; 360] = std::array::from_fn(|_| 0);

        for ind in 0..values.len() {
            let pivot_angle = start_angle + 45 * ind as u16;
            let allowed_range = pivot_angle_to_allowed_range(pivot_angle, zone_allowed_range)?;
            let range_to_value = Self::gen_range(pivot_angle, allowed_range, &values[ind]);
            for (angle, value) in range_to_value {
                if angle >= 360 || angle < 0 {
                    bail!("Incorrectly generated angle '{}'", angle)
                }
                let angle = angle as usize;
                if angle_to_value[angle].is_some() {
                    bail!("Duplicate angle '{}'", angle)
                }
                angle_to_value[angle] = Some(value.clone());
                angle_to_zone[angle] = ind as u8;
            }
        }

        // for ind in 0..angle_to_value.len(){
        //     let value = match angle_to_value[ind].clone() {
        //         None => {"None".to_string()}
        //         Some(value) => {
        //             let mut combined_str = String::new();
        //             for val in value{
        //                 combined_str = combined_str.add(format!("{} ", val).as_str());
        //             }
        //             combined_str
        //         }
        //     };
        //     debug!("{}: {}", ind, value)
        // }

        Ok(Self {
            angle_to_value,
            angle_to_zone,
            threshold,
            prev_zone: None,
            prev_value: None,
        })
    }

    pub fn gen_range(pivot_angle: u16, allowed_range: u16, value: &Vec<T>) -> Vec<(u16, &Vec<T>)> {
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
