use crate::{
    io::{Action, Angle, Point, SensorInput},
    sig::Signal,
    sm::{StateFullMachine, StateMachine},
    sm_course::{delay, scale, wire},
};
use safer_ffi::prelude::*;
use std::ops::{Add, Mul};
pub mod io;
pub mod sig;
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

fn rotate((goal, (position, theta)): (Point, (Point, Angle))) -> Action {
    Action {
        fvel: 0.0,
        rvel: 2.0 * (position.angle_to(goal) - theta).0,
    }
}

fn forward((goal, (position, _)): (Point, (Point, Angle))) -> Action {
    Action {
        fvel: 2.0 * position.distance(goal),
        rvel: 0.0,
    }
}

struct FollowFigure {
    fig: Vec<Point>,
}

impl StateMachine<Point, Point> for FollowFigure {
    type State = usize;

    fn start_state(&self) -> Self::State {
        0
    }

    fn done(&self, state: Self::State) -> bool {
        state == self.fig.len()
    }

    fn next_values(
        &self,
        mut idx: Self::State,
        input: Option<Point>,
    ) -> (Self::State, Option<Point>) {
        if input
            .zip(self.fig.get(idx))
            .map(|(p1, p2)| p1.is_near(p2.clone(), 0.02))
            .unwrap_or(false)
        {
            idx += 1;
        }
        (idx, self.fig.get(idx).cloned())
    }
}

#[ffi_export]
fn sm_simple(_incr: f64) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    let dynamic_move_to_point = forward.switch(rotate, |(goal, (position, theta))| {
        theta.is_near(position.angle_to(goal), 0.02)
    });
    let goal_generator = (|sensors: SensorInput| sensors.odometry.pos).cascade(FollowFigure {
        fig: vec![
            Point::new(0.5, 0.5),
            Point::new(0.0, 1.0),
            Point::new(-0.5, 0.5),
            Point::new(0.0, 0.0),
        ],
    });

    let sm = goal_generator
        .parallel(|sensors: SensorInput| (sensors.odometry.pos, Angle::new(sensors.odometry.theta)))
        .cascade(dynamic_move_to_point);

    Box::new(StateFullMachineOpaque {
        sfm: Box::new(sm.into_state_full_machine()),
    })
    .into()
}

#[ffi_export]
fn sm_step(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>, input: SensorInput) -> Action {
    sm.sfm.step(Some(input)).unwrap_or_default()
}

#[ffi_export]
fn sm_is_done(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>) -> bool {
    sm.sfm.is_done()
}

#[ffi_export]
fn sm_reset(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>) {
    sm.sfm.reset()
}

#[derive_ReprC]
#[repr(opaque)]
pub struct SignalOpaque<O>
where
    O: ReprC,
{
    sig: Box<dyn Signal<Out = O>>,
}

#[ffi_export]
fn sig_unit() -> repr_c::Box<SignalOpaque<f64>> {
    Box::new(SignalOpaque {
        sig: Box::new(sig::unit()),
    })
    .into()
}

#[ffi_export]
fn sig_cos(omega: f64, theta: f64) -> repr_c::Box<SignalOpaque<f64>> {
    Box::new(SignalOpaque {
        sig: Box::new(sig::cosine(omega, theta)),
    })
    .into()
}

#[ffi_export]
fn sig() -> repr_c::Box<SignalOpaque<f64>> {
    let sm = delay(0.0)
        .parallel(scale(-1.0).cascade(delay(0.0)).cascade(delay(0.0)))
        .cascade(|(f1, f2)| f1 + f2);
    Box::new(SignalOpaque {
        sig: Box::new(sm.transduce_signal(sig::unit())),
    })
    .into()
}

#[ffi_export]
fn sig_sample(s: &'_ mut SignalOpaque<f64>, n: i32) -> f64 {
    s.sig.sample(n)
}

#[cfg(feature = "headers")]
pub fn generate_headers() -> ::std::io::Result<()> {
    use safer_ffi::headers::Language;

    ::safer_ffi::headers::builder()
        .with_language(Language::Python)
        .to_file(format!("{}.h", env!("CARGO_PKG_NAME")))?
        .generate()
}
