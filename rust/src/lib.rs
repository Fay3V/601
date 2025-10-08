use crate::{
    sm::{StateFullMachine, StateMachine},
    sm_course::{Delay, Gain},
};
use safer_ffi::prelude::*;
use std::ops::{Add, Mul};
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

#[ffi_export]
fn sm_world() -> repr_c::Box<StateFullMachineOpaque<f64, f64>> {
    const K: f64 = -1.5;
    const D: f64 = 1.0;

    struct WallController;
    impl StateMachine<f64> for WallController {
        type State = ();
        type Output = f64;

        fn start_state(&self) -> Self::State {
            ()
        }

        fn next_values(
            &self,
            _state: Self::State,
            input: Option<f64>,
        ) -> (Self::State, Option<Self::Output>) {
            ((), input.map(|i| K * (D - i)))
        }
    }

    const DELTA: f64 = 0.1;
    struct WallWorld;
    impl StateMachine<f64> for WallWorld {
        type State = Option<f64>;
        type Output = f64;

        fn start_state(&self) -> Self::State {
            Some(5.0)
        }

        fn next_values(
            &self,
            state: Self::State,
            input: Option<f64>,
        ) -> (Self::State, Option<Self::Output>) {
            (
                input.zip(state).map(|(input, state)| state - DELTA * input),
                state,
            )
        }
    }

    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            WallController
                .cascade(WallWorld)
                .feedback()
                .into_state_full(),
        ),
    })
    .into()
}

#[ffi_export]
fn sm_factorial() -> repr_c::Box<StateFullMachineOpaque<f64, f64>> {
    let fac = Delay::new(1.0).feedback_op(Gain::new(1.0), f64::mul);
    let counter = Delay::new(1.0).feedback_op(Gain::new(1.0), f64::add);
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(counter.cascade(fac).into_state_full()),
    })
    .into()
}

#[ffi_export]
fn sm_counter() -> repr_c::Box<StateFullMachineOpaque<f64, f64>> {
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            Delay::new(1.0)
                .feedback_op(Gain::new(1.0), f64::add)
                .into_state_full(),
        ),
    })
    .into()
}

#[ffi_export]
fn sm_step(sm: &'_ mut StateFullMachineOpaque<f64, f64>, input: f64) -> f64 {
    sm.sfm
        .step(Some(input))
        .expect("cannot step the state machine")
}

#[ffi_export]
fn sm_run(sm: &'_ mut StateFullMachineOpaque<f64, f64>, n: usize) -> repr_c::Vec<f64> {
    sm.sfm.run(n).into()
}

#[ffi_export]
fn sm_reset(sm: &'_ mut StateFullMachineOpaque<f64, f64>) {
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
