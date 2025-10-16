use std::{f64::consts::PI, ops::Sub};

use safer_ffi::derive_ReprC;

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub fn distance(&self, other: Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    pub fn is_near(&self, other: Point, epsilon: f64) -> bool {
        self.distance(other) < epsilon
    }

    pub fn angle_to(&self, other: Point) -> Angle {
        let dy = other.y - self.y;
        let dx = other.x - self.x;
        Angle::new(dy.atan2(dx))
    }
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Pose {
    pub pos: Point,
    pub theta: f64,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct SensorInput {
    pub sonars: [f64; 8],
    pub odometry: Pose,
    // analog_inputs: [f64; 4],
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone, Default)]
pub struct Action {
    pub fvel: f64,
    pub rvel: f64,
    // voltage: f64,
}
#[derive(Debug, Clone, Copy)]
pub struct Angle(pub(crate) f64);

impl Angle {
    pub fn new(value: f64) -> Self {
        Self((value + PI).rem_euclid(2.0 * PI) - PI)
    }
    pub fn is_near(self, other: Angle, epsilon: f64) -> bool {
        (self - other).0.abs() < epsilon
    }
}

impl Sub<Angle> for Angle {
    type Output = Angle;

    fn sub(self, rhs: Angle) -> Self::Output {
        Angle::new(self.0 - rhs.0)
    }
}

impl Sub<Angle> for f64 {
    type Output = f64;

    fn sub(self, rhs: Angle) -> Self::Output {
        self - rhs.0
    }
}
