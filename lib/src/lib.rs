use crate::{
    io::{Action, Angle, SensorInput},
    sf::SystemFunction,
    sig::{Signal, constant},
    sm::{StateFullMachine, StateMachine},
    sonars::get_distance_right,
};
use safer_ffi::prelude::*;
use std::cell::Cell;
pub mod io;

pub mod poly;

pub mod sf;

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

fn controller(desired_d: f64, k: f64) -> impl StateMachine<(u32, f64), Action> {
    (move |(steps, ds): (u32, f64)| (steps, desired_d - ds))
        .cascade(move |(steps, err)| {
            eprintln!("err[{}]={err}", steps);
            (steps, k * err)
        })
        .cascade(|(steps, rvel)| {
            eprintln!("w[{}]={rvel}", steps);
            Action { fvel: V, rvel }
        })
}

fn sensor() -> impl StateMachine<(u32, SensorInput), (u32, f64)> {
    |(steps, sensors): (u32, SensorInput)| {
        let dist = get_distance_right(&sensors.sonars);
        eprintln!("do[{}]={dist}", steps - 1);
        eprintln!(
            "theta[{}]={}",
            steps - 1,
            Angle::new(sensors.odometry.theta).0
        );
        (steps, dist)
    }
}

#[ffi_export]
fn sm(desired_d: f64, k: f64) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    dbg!((desired_d, k));
    let steps = Cell::new(0);
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            (move |input| {
                let s = steps.get() + 1;
                steps.set(s);
                (s, input)
            })
            .cascade(sensor().cascade(controller(desired_d, k)))
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

pub fn wall_follower_model(k: f64, t: f64, v: f64) -> SystemFunction {
    // let numerator = Poly::new([k * v * t * t, 0.0, 0.0]);
    // let denominator = Poly::new([1.0 + k * v * t * t, -2.0, 1.0]);
    // let sf = SystemFunction::new(numerator, denominator);
    // println!("1: {sf}");
    // println!("===============================");

    let controller = sf::gain(k);
    let plant1 = sf::gain(t)
        .cascade(sf::delay())
        .cascade(sf::gain(1.0).feedback_add(Some(sf::delay())));
    let plant2 = sf::gain(v * t)
        .cascade(sf::delay())
        .cascade(sf::gain(1.0).feedback_add(Some(sf::delay())));
    let sf = controller
        .cascade(plant1)
        .cascade(plant2)
        .feedback_sub(None);
    // println!("2: {sf}");
    sf
}

#[ffi_export]
fn sig(init_d: f64, k: f64, desired_d: f64) -> repr_c::Box<SignalOpaque<f64>> {
    Box::new(SignalOpaque {
        sig: Box::new(
            wall_follower_model(k, T, V)
                .into_sm(
                    Some(vec![desired_d, desired_d]),
                    // Some(vec![0.5004870506048704, 0.5031447799628332]),
                    Some(vec![0.5034640638578065, 0.49970114809773036]),
                    // Some(vec![0.50, 0.5031447799628332]),
                )
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
