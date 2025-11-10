use crate::{
    io::{Action, Angle, SensorInput},
    sf::SystemFunction,
    sig::{Signal, constant},
    sm::{StateFullMachine, StateMachine},
    sm_course::delay,
    sonars::get_distance_right,
};
use safer_ffi::{option::TaggedOption, prelude::*};
use std::cell::Cell;
pub mod io;

pub mod poly;

pub mod sf;

pub mod opt;
pub mod sig;
pub mod sm;
pub mod sm_course;
pub mod sonars;

#[derive_ReprC]
#[repr(opaque)]
pub struct StateFullMachineOpaque<I, O>
where
    I: ReprC,
    O: ReprC,
{
    sfm: Box<dyn StateFullMachine<I, O>>,
}

const V: f64 = 0.1;
const T: f64 = 0.1;

fn controller(desired_d: f64, k3: f64, k4: f64) -> impl StateMachine<(f64, Option<f64>), Action> {
    (move |(ds, angle): (f64, Option<f64>)| angle.map(|angle| k3 * (desired_d - ds) - k4 * angle))
        .cascade(|rvel: Option<f64>| Action {
            fvel: V,
            rvel: rvel.unwrap_or(0.0),
        })
}

fn sensor() -> impl StateMachine<AnglePropInput, (f64, Option<f64>)> {
    |sonars: AnglePropInput| (sonars.distance, sonars.angle.into_rust())
}

#[derive_ReprC]
#[repr(C)]
#[derive(Debug, Clone)]
pub struct AnglePropInput {
    distance: f64,
    angle: TaggedOption<f64>,
}

#[ffi_export]
fn sm(
    desired_d: f64,
    k1: f64,
    k2: f64,
) -> repr_c::Box<StateFullMachineOpaque<AnglePropInput, Action>> {
    dbg!((desired_d, (k1, k2)));
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            sensor()
                .cascade(controller(desired_d, k1, k2))
                .into_state_full_machine(),
        ),
    })
    .into()
}

#[ffi_export]
fn sm_step(
    sm: &'_ mut StateFullMachineOpaque<AnglePropInput, Action>,
    input: AnglePropInput,
) -> Action {
    sm.sfm.step(Some(input)).unwrap_or_default()
}

#[ffi_export]
fn sm_is_done(sm: &'_ mut StateFullMachineOpaque<AnglePropInput, Action>) -> bool {
    sm.sfm.is_done()
}

#[ffi_export]
fn sm_reset(sm: &'_ mut StateFullMachineOpaque<AnglePropInput, Action>) {
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

pub fn delay_plus_prop_model(k1: f64, k2: f64) -> SystemFunction {
    let controller = sf::gain(k1).feedforward_add(Some(sf::gain(k2).cascade(sf::delay())));

    let plant1 = sf::gain(T)
        .cascade(sf::delay())
        .cascade(sf::gain(1.0).feedback_add(Some(sf::delay())));
    let plant2 = sf::gain(V * T)
        .cascade(sf::delay())
        .cascade(sf::gain(1.0).feedback_add(Some(sf::delay())));
    let sf = controller
        .cascade(plant1)
        .cascade(plant2)
        .feedback_sub(None);
    println!("{sf}");
    sf
}

pub fn angle_plus_prop_model(k3: f64, k4: f64) -> SystemFunction {
    let plant1 = sf::gain(T)
        .cascade(sf::delay())
        .cascade(sf::gain(1.0).feedback_add(Some(sf::delay())));

    let plant2 = sf::gain(V * T)
        .cascade(sf::delay())
        .cascade(sf::gain(1.0).feedback_add(Some(sf::delay())));
    let sf = sf::gain(k3)
        .cascade(plant1.feedback_sub(Some(sf::gain(k4))))
        .cascade(plant2)
        .feedback_sub(None);
    // println!("{sf}");
    sf
}

#[ffi_export]
fn sig(k3: f64, k4: f64, desired_d: f64) -> repr_c::Box<SignalOpaque<f64>> {
    Box::new(SignalOpaque {
        sig: Box::new(
            angle_plus_prop_model(k3, k4)
                .into_sm(Some(vec![desired_d, desired_d]), Some(vec![0.503, 0.499]))
                .transduce_signal(constant(desired_d)),
        ),
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
