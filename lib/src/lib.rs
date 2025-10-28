use crate::{
    io::{Action, SensorInput},
    sig::{Signal, constant},
    sm::{StateFullMachine, StateMachine},
    sm_course::{delay, scale, wire},
};
use safer_ffi::prelude::*;
use std::ops::Add;
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

fn controller(desired_d: f64, k: f64) -> impl StateMachine<f64, Action> {
    (move |ds: f64| desired_d - ds)
        .cascade(scale(k))
        .cascade(|fvel| dbg!(Action::foward(fvel)))
}

fn sensor(init_d: f64) -> impl StateMachine<SensorInput, f64> {
    (|sensors: SensorInput| sensors.sonars[3]).cascade(delay(init_d))
}

#[ffi_export]
fn sm(
    init_d: f64,
    desired_d: f64,
    k: f64,
) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            sensor(init_d)
                .cascade(controller(desired_d, k))
                .into_state_full_machine(),
        ),
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
fn sig(t: f64, init_d: f64, k: f64, desired_d: f64) -> repr_c::Box<SignalOpaque<f64>> {
    fn sm_controller(k: f64) -> impl StateMachine<f64, f64> {
        scale(k)
    }

    fn sm_plant(t: f64, init_d: f64) -> impl StateMachine<f64, f64> {
        delay(0.0)
            .cascade(scale(-t))
            .cascade(wire().feedback_op(delay(init_d), f64::add))
    }

    fn sm_sensor(init_d: f64) -> impl StateMachine<f64, f64> {
        delay(init_d)
    }

    fn sm_wall_finder(t: f64, init_d: f64, k: f64) -> impl StateMachine<f64, f64> {
        sm_controller(k)
            .cascade(sm_plant(t, init_d))
            .feedback_op(sm_sensor(init_d), |f1, f2| f1 - f2)
    }

    Box::new(SignalOpaque {
        sig: Box::new(sm_wall_finder(t, init_d, k).transduce_signal(constant(desired_d))),
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
