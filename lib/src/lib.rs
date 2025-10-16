use crate::{
    io::{Action, Angle, Point, SensorInput},
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

fn rotate(input: (Point, (Point, Angle))) -> Action {
    let (goal, (position, theta)) = input;
    Action {
        fvel: 0.0,
        rvel: 2.0 * (position.angle_to(goal) - theta).0,
    }
}

fn forward(input: (Point, (Point, Angle))) -> Action {
    let (goal, (position, _)) = input;
    Action {
        fvel: 2.0 * position.distance(goal),
        rvel: 0.0,
    }
}

struct FollowFigure<SM> {
    fig: Vec<Point>,
    sm: SM,
}

impl<SM, I> StateMachine<I> for FollowFigure<SM>
where
    I: Clone,
    SM: StateMachine<(Point, I)>,
    SM::State: Clone,
{
    type State = (usize, SM::State);
    type Output = SM::Output;

    fn start_state(&self) -> Self::State {
        (0, self.sm.start_state())
    }

    fn done(&self, state: Self::State) -> bool {
        (state.0 == self.fig.len() - 1) && self.sm.done(state.1.clone())
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        let input = input.expect("no input");
        let mut idx = state.0;
        if self.sm.done(state.1.clone()) && idx < self.fig.len() - 1 {
            idx += 1;
        }
        let (new_state, out) = self.sm.next_values(state.1, Some((self.fig[idx], input)));
        ((idx, new_state), out)
    }
}

#[ffi_export]
fn sm_simple(_incr: f64) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            FollowFigure {
                fig: vec![
                    Point::new(0.5, 0.5),
                    Point::new(0.0, 1.0),
                    Point::new(-0.5, 0.5),
                    Point::new(0.0, 0.0),
                ],
                sm: (|(goal, sensors): (Point, SensorInput)| {
                    (
                        goal,
                        (sensors.odometry.pos, Angle::new(sensors.odometry.theta)),
                    )
                })
                .cascade(
                    forward
                        .switch(
                            rotate,
                            |(goal, (position, theta)): (Point, (Point, Angle))| {
                                theta.is_near(position.angle_to(goal), 0.02)
                            },
                        )
                        .until(|(goal, (position, _)): (Point, (Point, Angle))| {
                            position.is_near(goal, 0.02)
                        }),
                ),
            }
            .into_state_full(),
        ),
    })
    .into()
}

#[ffi_export]
fn sm_step(sm: &'_ mut StateFullMachineOpaque<SensorInput, Action>, input: SensorInput) -> Action {
    sm.sfm.step(Some(input)).unwrap_or_default()
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
