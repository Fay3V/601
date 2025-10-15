use crate::{
    io::{Action, Angle, Position, SensorInput},
    sm::{StateFullMachine, StateMachine},
};
use safer_ffi::prelude::*;
pub mod io;
pub mod sm;
pub mod sm_course;

#[derive_ReprC]
#[repr(opaque)]
pub struct StateFullMachineOpaque<I, O>
where
    I: ReprC,
    O: ReprC,
{
    sfm: Box<dyn StateFullMachine<I, O>>,
}

struct Rotate {
    heading_delta: f64,
    rotation_gain: f64,
    angle_epsilon: f64,
}

impl StateMachine<SensorInput> for Rotate {
    type State = Option<(f64, Angle)>;
    type Output = Action;

    fn start_state(&self) -> Self::State {
        None
    }

    fn done(&self, state: Self::State) -> bool {
        state
            .map(|(theta_error, _)| theta_error.abs() < self.angle_epsilon)
            .unwrap_or(false)
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<SensorInput>,
    ) -> (Self::State, Option<Self::Output>) {
        let input = input.expect("input value");
        let theta_curr = Angle::new(input.odometry.theta);
        let theta_error = state
            .map(|(theta_error, theta_last)| theta_error - (theta_curr - theta_last))
            .unwrap_or(self.heading_delta);
        let action = Action {
            fvel: 0.0,
            rvel: self.rotation_gain * theta_error,
        };
        (Some((theta_error, theta_curr)), Some(action))
    }
}

struct Forward {
    delta_desired: f64,
    forward_gain: f64,
    dist_target_epsilon: f64,
}

impl StateMachine<SensorInput> for Forward {
    type State = Option<(Position, Position)>;
    type Output = Action;

    fn start_state(&self) -> Self::State {
        None
    }

    fn done(&self, state: Self::State) -> bool {
        state
            .map(|(start_pos, last_pos)| {
                (start_pos.distance(last_pos) - self.delta_desired).abs() < self.dist_target_epsilon
            })
            .unwrap_or(false)
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<SensorInput>,
    ) -> (Self::State, Option<Self::Output>) {
        let input = input.expect("input value");
        let curr_pos = input.odometry.pos;
        let start_pos = state.map(|(start_pos, _)| start_pos).unwrap_or(curr_pos);
        let action = Action {
            fvel: self.forward_gain * (self.delta_desired - start_pos.distance(curr_pos)),
            rvel: 0.0,
        };
        (Some((start_pos, curr_pos)), Some(action))
    }
}

#[ffi_export]
fn sm_simple(heading_delta: f64) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            // Rotate {
            //     heading_delta,
            //     rotation_gain: 0.5,
            //     angle_epsilon: 0.01,
            // }
            // .into_state_full(),
            Forward {
                delta_desired: heading_delta,
                forward_gain: 2.0,
                dist_target_epsilon: 0.02,
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
