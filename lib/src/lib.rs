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

struct XYDriver {
    forward_gain: f64,
    rotation_gain: f64,
    angle_epsilon: f64,
    distance_epsilon: f64,
}

impl StateMachine<(Option<Position>, SensorInput)> for XYDriver {
    type State = bool;
    type Output = Action;

    fn start_state(&self) -> Self::State {
        false
    }

    fn done(&self, state: Self::State) -> bool {
        state
    }

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<(Option<Position>, SensorInput)>,
    ) -> (Self::State, Option<Self::Output>) {
        let (goal_pos, sensors) = input.expect("no input");
        let robot_pos = sensors.odometry.pos;
        let robot_theta = Angle::new(sensors.odometry.theta);

        if let Some(goal_pos) = goal_pos {
            let heading_theta = robot_pos.angle_to(goal_pos);
            if robot_theta.is_near(heading_theta, self.angle_epsilon) {
                let distance_error = robot_pos.distance(goal_pos);
                if distance_error < self.distance_epsilon {
                    (true, Some(Action::default()))
                } else {
                    (
                        false,
                        Some(Action {
                            fvel: distance_error * self.forward_gain,
                            rvel: 0.0,
                        }),
                    )
                }
            } else {
                let heading_error = heading_theta - robot_theta;
                (
                    false,
                    Some(Action {
                        fvel: 0.0,
                        rvel: heading_error.0 * self.rotation_gain,
                    }),
                )
            }
        } else {
            (true, Some(Action::default()))
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Direction {
    North,
    South,
    East,
    West,
}

struct SpyroGyra {
    distance_epsilon: f64,
    incr: f64,
}

impl StateMachine<SensorInput> for SpyroGyra {
    type State = (Direction, f64, Option<Position>);
    type Output = (Option<Position>, SensorInput);

    fn start_state(&self) -> Self::State {
        (Direction::South, 0.0, None)
    }

    // fn done(&self, state: Self::State) -> bool {
    //     // self.xy.done(state)
    //     false
    // }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<SensorInput>,
    ) -> (Self::State, Option<Self::Output>) {
        let input = input.expect("no input");
        let (direction, length, sub_goal) = state;
        let robot_pos = input.odometry.pos;
        let mut sub_goal = sub_goal.unwrap_or(robot_pos);
        let (direction, length, sub_goal) = if robot_pos.is_near(sub_goal, self.distance_epsilon) {
            let length = length + self.incr;
            let direction = match direction {
                Direction::North => {
                    sub_goal.x -= length;
                    Direction::West
                }
                Direction::South => {
                    sub_goal.x += length;
                    Direction::East
                }
                Direction::East => {
                    sub_goal.y += length;
                    Direction::North
                }
                Direction::West => {
                    sub_goal.y -= length;
                    Direction::South
                }
            };
            (direction, length, sub_goal)
        } else {
            (direction, length, sub_goal)
        };
        (
            (direction, length, Some(sub_goal)),
            Some((Some(sub_goal), input)),
        )
    }
}

#[ffi_export]
fn sm_simple(incr: f64) -> repr_c::Box<StateFullMachineOpaque<SensorInput, Action>> {
    Box::new(StateFullMachineOpaque {
        sfm: Box::new(
            SpyroGyra {
                distance_epsilon: 0.02,
                incr,
            }
            .cascade(XYDriver {
                forward_gain: 2.0,
                rotation_gain: 2.0,
                angle_epsilon: 0.05,
                distance_epsilon: 0.02,
            })
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
