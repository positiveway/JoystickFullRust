use color_eyre::owo_colors::colors::xterm::CodGray;
use serde::{Deserialize, Serialize};
use crate::mouse::Coords;

fn smoothing_factor(t_e: f64, cutoff: f64) -> f64 {
    let r = 2.0 * std::f64::consts::PI * cutoff * t_e;
    r / (r + 1.0)
}

fn exponential_smoothing(a: f64, x: f64, x_prev: f64) -> f64 {
    a * x + (1.0 - a) * x_prev
}

fn create_filter(cutoff: f64, beta: f64) -> impl FnMut(f64, f64) -> f64 {
    let mut self_filter = Filter {
        cutoff: cutoff,
        beta: beta,
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

pub fn hypot<T>(a: T, b: T) -> f64
    where T: core::ops::Mul<T, Output=T> + core::ops::Add<T, Output=T> + core::convert::Into<f64> + Copy
{
    (a * a + b * b).into().sqrt()
}

const RADIANS_TO_DEGREES: f32 = 180f32 / std::f32::consts::PI;
const DEGREES_TO_RADIANS: f32 = std::f32::consts::PI / 180f32;

pub fn resolve_angle(angle: f32) -> f32 {
    ((angle + 360.0) % 360.0).round()
}


#[derive(PartialEq, Copy, Clone, Default, Debug, Serialize, Deserialize)]
pub struct Vector {
    pub x: f32,
    pub y: f32,
}

impl Vector {
    pub fn as_coords(&self) -> Coords {
        Coords {
            x: Some(self.x),
            y: Some(self.y),
        }
    }
    pub fn from_2_coords(point1: Coords, point2: Coords) -> Option<Vector> {
        if point1.any_is_none() || point2.any_is_none() {
            return None;
        }

        Some(Self {
            x: point2.x.unwrap() - point1.x.unwrap(),
            y: point2.y.unwrap() - point1.y.unwrap(),
        })
    }

    pub fn from_coords(coords: Coords) -> Self {
        Self {
            x: coords.x.unwrap(),
            y: coords.y.unwrap(),
        }
    }

    pub fn angle(&self) -> f32 {
        let angle_in_radians = self.y.atan2(self.x);
        let angle_in_degrees = angle_in_radians * RADIANS_TO_DEGREES;
        resolve_angle(angle_in_degrees)
    }

    pub fn distance(&self) -> f32 {
        self.x.hypot(self.y)
    }

    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
        }
    }
}

impl std::ops::Add<Vector> for Vector {
    type Output = Vector;

    fn add(self, other: Vector) -> Vector {
        Vector {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl std::ops::AddAssign<Vector> for Vector {
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl std::ops::Sub<Vector> for Vector {
    type Output = Vector;

    fn sub(self, other: Vector) -> Vector {
        Vector {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl std::ops::SubAssign<Vector> for Vector {
    fn sub_assign(&mut self, other: Vector) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

pub fn rotate_by_angle(point1: Coords, point2: Coords, rotation_angle: f32) -> Option<Vector> {
    if point1.any_is_none() || point2.any_is_none() {
        return None;
    }

    let point1 = Vector::from_coords(point1);
    let mut point2 = Vector::from_coords(point2);


    let rotation_angle = rotation_angle * DEGREES_TO_RADIANS;
    let sin: f32 = rotation_angle.sin();
    let cos: f32 = rotation_angle.cos();

    point2 -= point1;


    let mut rotated_point = Vector {
        x: point2.x * cos - point2.y * sin,
        y: point2.x * sin + point2.y * cos,
    };

    rotated_point += point1;

    Some(rotated_point)
}

pub fn rotate_around_center(point: Coords, rotation_angle: f32) -> Option<Vector> {
    rotate_by_angle(Vector::zero().as_coords(), point, rotation_angle)
}