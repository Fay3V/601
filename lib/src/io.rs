use std::{f64::consts::PI, ops::Sub};

use safer_ffi::derive_ReprC;

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Pose {
    pub pos: Position,
    pub theta: f64,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct SensorInput {
    pub sonars: [f64; 8],
    pub odometry: Pose,
    // analog_inputs: [f64; 4],
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
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
