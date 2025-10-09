use crate::{
    sm::{StateFullMachine, StateMachine},
    sm_course::{Delay, Gain},
};
use rand::Rng;
use safer_ffi::prelude::*;
use std::{
    cell::Cell,
    f64::{self, consts::PI},
    ops::{Add, Mul},
};
pub mod sm;
pub mod sm_course;
pub mod util;

#[derive_ReprC]
#[repr(opaque)]
pub struct StateFullMachineOpaque<I, O>
where
    I: ReprC,
    O: ReprC,
{
    sfm: Box<dyn StateFullMachine<I, O>>,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
struct Pose {
    x: f64,
    y: f64,
    theta: f64,
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
struct SensorInput {
    sonars: [f64; 8],
    odometry: Pose,
    // analog_inputs: [f64; 4],
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
struct Action {
    fvel: f64,
    rvel: f64,
    // voltage: f64,
}

struct Rotate {
    heading_delta: f64,
    rotation_gain: f64,
    angle_epsilon: f64,
}

impl StateMachine<SensorInput> for Rotate {
    type State = Option<(f64, f64)>;
    type Output = Action;

    fn start_state(&self) -> Self::State {
        None
    }

    fn done(&self, state: Self::State) -> bool {
        state
            .map(|(theta_acc, _)| (self.heading_delta - theta_acc).abs() < self.angle_epsilon)
            .unwrap_or(false)
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<SensorInput>,
    ) -> (Self::State, Option<Self::Output>) {
        let input = input.expect("input value");
        let curr_theta = input.odometry.theta;
        let theta_acc = state
            .map(|(theta_acc, theta_last)| {
                theta_acc + util::fix_angle_plus_minus_pi(curr_theta - theta_last)
            })
            .unwrap_or(0.0);
        let action = Action {
            fvel: 0.0,
            rvel: self.rotation_gain * (self.heading_delta - theta_acc),
        };
        (Some((theta_acc, curr_theta)), Some(action))
    }
}

#[ffi_export]
fn sm_simple(heading_delta: f64) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            Rotate {
                heading_delta,
                rotation_gain: 0.5,
                angle_epsilon: 0.01,
            }
            .into_state_full(),
        ),
    })
    .into()
}

#[ffi_export]
fn sm_step(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>, input: SensorInput) -> Action {
    sm.sfm
        .step(Some(input))
        .expect("cannot step the state machine")
}

#[ffi_export]
fn sm_run(
    sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>,
    n: usize,
) -> repr_c::Vec<Action> {
    sm.sfm.run(Some(n)).into()
}

#[ffi_export]
fn sm_is_done(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>) -> bool {
    sm.sfm.is_done()
}

#[ffi_export]
fn sm_reset(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>) {
    sm.sfm.reset()
}

#[cfg(feature = "headers")]
pub fn generate_headers() -> ::std::io::Result<()> {
    use safer_ffi::headers::Language;

    ::safer_ffi::headers::builder()
        .with_language(Language::Python)
        .to_file(format!("{}.h", env!("CARGO_PKG_NAME")))?
        .generate()
}
