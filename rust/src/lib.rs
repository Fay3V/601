use crate::{
    sm::StateMachine,
    sm_course::{Delay, Incr},
};
use safer_ffi::prelude::*;

pub mod sm;
pub mod sm_course;

trait StateFullMachine {
    fn reset(&mut self);
    fn step(&mut self, input: i32) -> Option<i32>;
}

impl<SM> StateFullMachine for (SM::State, SM)
where
    SM: StateMachine<i32, Output = i32>,
{
    fn reset(&mut self) {
        self.0 = self.1.start_state();
    }

    fn step(&mut self, input: i32) -> Option<i32> {
        self.1.step(&mut self.0, Some(input))
    }
}

#[derive_ReprC]
#[repr(opaque)]
pub struct StateFullMachineOpaque {
    sfm: Box<dyn StateFullMachine>,
}

#[ffi_export]
fn sm_new() -> repr_c::Box<StateFullMachineOpaque> {
    let sm = Incr::<1>.cascade(Delay::new(0)).feedback();
    let state = sm.start_state();
    Box::new(StateFullMachineOpaque {
        sfm: Box::new((state, sm)),
    })
    .into()
}

#[ffi_export]
fn sm_step(sm: &'_ mut StateFullMachineOpaque, input: i32) -> i32 {
    sm.sfm.step(input).unwrap_or_default()
}

#[ffi_export]
fn sm_reset(sm: &'_ mut StateFullMachineOpaque) {
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
