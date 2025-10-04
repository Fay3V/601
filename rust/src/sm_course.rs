use crate::sm::StateMachine;
use core::fmt;
use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

pub struct Incr<const STEP: i32>;
impl<const STEP: i32> StateMachine<i32> for Incr<STEP> {
    type State = ();
    type Output = i32;

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<i32>,
    ) -> (Self::State, Option<Self::Output>) {
        ((), input.map(|input| input + STEP))
    }

    fn start_state(&self) -> Self::State {
        ()
    }
}

#[derive(Default)]
pub struct Delay<I> {
    val: I,
}

impl<I> Delay<I> {
    pub fn new(val: I) -> Self {
        Self { val }
    }
}

impl<I: Clone + fmt::Debug> StateMachine<I> for Delay<I> {
    type State = Option<I>;
    type Output = I;

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        (input, state)
    }

    fn start_state(&self) -> Self::State {
        Some(self.val.clone())
    }
}

#[derive(Default)]
pub struct Adder<Lhs, Rhs> {
    _phantom: PhantomData<(Lhs, Rhs)>,
}

impl<Lhs, Rhs, O> StateMachine<(Lhs, Rhs)> for Adder<Lhs, Rhs>
where
    Lhs: Clone,
    Rhs: Clone,
    Lhs: Add<Rhs, Output = O>,
{
    type State = ();
    type Output = O;

    fn start_state(&self) -> Self::State {
        ()
    }

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<(Lhs, Rhs)>,
    ) -> (Self::State, Option<Self::Output>) {
        let output = input.map(|(l, r)| l + r);
        ((), output)
    }
}

#[derive(Default)]
pub struct Multiplier<Lhs, Rhs> {
    _phantom: PhantomData<(Lhs, Rhs)>,
}

impl<Lhs, Rhs, O> StateMachine<(Lhs, Rhs)> for Multiplier<Lhs, Rhs>
where
    Lhs: Clone,
    Rhs: Clone,
    Lhs: Mul<Rhs, Output = O>,
{
    type State = ();
    type Output = O;

    fn start_state(&self) -> Self::State {
        ()
    }

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<(Lhs, Rhs)>,
    ) -> (Self::State, Option<Self::Output>) {
        let output = input.map(|(l, r)| l * r);
        ((), output)
    }
}

#[derive(Default)]
pub struct Constant<const VAL: i32>;
impl<const VAL: i32> StateMachine<i32> for Constant<VAL> {
    type State = ();
    type Output = i32;

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<i32>,
    ) -> (Self::State, Option<Self::Output>) {
        ((), input.map(|_input| VAL))
    }

    fn start_state(&self) -> Self::State {
        ()
    }
}

pub type Wire = Gain<i32, 1>;

#[derive(Default)]
pub struct Gain<I, const G: i32 = 1> {
    _phantom: PhantomData<I>,
}

impl<I, const G: i32> StateMachine<I> for Gain<I, G>
where
    I: Clone,
    I: Mul<i32>,
{
    type State = ();
    type Output = <I as Mul<i32>>::Output;

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        ((), input.map(|input| input * G))
    }

    fn start_state(&self) -> Self::State {
        ()
    }
}

#[test]
fn it_works() {
    struct Delay2<I> {
        val0: I,
        val1: I,
    }

    impl<I> Delay2<I> {
        fn new(val0: I, val1: I) -> Self {
            Self { val0, val1 }
        }
    }

    impl<I: Clone + fmt::Debug> StateMachine<I> for Delay2<I> {
        type State = (Option<I>, Option<I>);
        type Output = I;

        fn next_values(
            &self,
            state: Self::State,
            input: Option<I>,
        ) -> (Self::State, Option<Self::Output>) {
            ((state.1, input), state.0)
        }

        fn start_state(&self) -> Self::State {
            (Some(self.val0.clone()), Some(self.val1.clone()))
        }
    }

    assert_eq!(
        &[100, 10, 1, 0, 2, 0, 0, 3, 0, 0],
        Delay2::new(100, 10)
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
    assert_eq!(
        &[10, 100, 0, 0, 0, 0, 0],
        Delay2::new(10, 100)
            .transduce([0, 0, 0, 0, 0, 0, 1])
            .as_slice()
    );
    assert_eq!(
        &[-1, 0, 1, 2, -3, 1],
        Delay2::new(-1, 0)
            .transduce([1, 2, -3, 1, 2, -3])
            .as_slice()
    );
    assert_eq!(
        &['a', 'b', 'c', 'd', 'e', 'f', 'g'],
        Delay2::new('a', 'b')
            .transduce(['c', 'd', 'e', 'f', 'g', 'i', 'j'])
            .as_slice()
    );
}

#[test]
fn test_accumulator() {
    struct Acc;

    impl StateMachine<i32> for Acc {
        type State = i32;
        type Output = i32;

        fn next_values(
            &self,
            state: Self::State,
            input: Option<i32>,
        ) -> (Self::State, Option<Self::Output>) {
            if let Some(input) = input {
                let acc = state + input;
                (acc, Some(acc))
            } else {
                (state, None)
            }
        }

        fn start_state(&self) -> Self::State {
            0
        }
    }

    assert_eq!(
        &[1, 1, 3, 3, 3, 6, 6, 6, 6, 10],
        Acc.transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4]).as_slice()
    );
}

#[test]
fn test_abc() {
    #[derive(Clone)]
    enum AbcState {
        ReadA,
        ReadB,
        ReadC,
        Stop,
    }

    struct Abc;

    impl StateMachine<char> for Abc {
        type State = AbcState;
        type Output = bool;

        fn next_values(
            &self,
            state: Self::State,
            input: Option<char>,
        ) -> (Self::State, Option<Self::Output>) {
            if let Some(input) = input {
                let (next_state, condition) = match state {
                    AbcState::ReadA => (AbcState::ReadB, input == 'a'),
                    AbcState::ReadB => (AbcState::ReadC, input == 'b'),
                    AbcState::ReadC => (AbcState::ReadA, input == 'c'),
                    AbcState::Stop => (AbcState::Stop, false),
                };
                (
                    if condition {
                        next_state
                    } else {
                        AbcState::Stop
                    },
                    Some(condition),
                )
            } else {
                (AbcState::Stop, None)
            }
        }

        fn start_state(&self) -> Self::State {
            AbcState::ReadA
        }
    }

    assert_eq!(
        &[true, false, false],
        Abc.transduce(['a', 'a', 'a']).as_slice()
    );
    assert_eq!(
        &[true, true, true, true, false, false, false],
        Abc.transduce(['a', 'b', 'c', 'a', 'c', 'b', 'a'])
            .as_slice()
    );
}

#[test]
fn test_average2() {
    struct Average2;

    impl StateMachine<i32> for Average2 {
        type State = Option<i32>;
        type Output = f32;

        fn next_values(
            &self,
            state: Self::State,
            input: Option<i32>,
        ) -> (Self::State, Option<Self::Output>) {
            let output = if let Some(state) = state
                && let Some(input) = input
            {
                Some((state as f32 + input as f32) / 2.0)
            } else {
                None
            };
            (input, output)
        }

        fn start_state(&self) -> Self::State {
            Some(0)
        }
    }
    assert_eq!(
        &[5.0, 7.5, 3.5, 6.0],
        Average2.transduce([10, 5, 2, 10]).as_slice()
    );
}

#[test]
fn test_cascade() {
    assert_eq!(
        &[100, 10, 1, 0, 2, 0, 0, 3, 0, 0],
        Delay::new(10)
            .cascade(Delay::new(100))
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
    assert_eq!(
        &[10, 100, 0, 0, 0, 0, 0],
        Delay::new(100)
            .cascade(Delay::new(10))
            .transduce([0, 0, 0, 0, 0, 0, 1])
            .as_slice()
    );
    assert_eq!(
        &[-1, 0, 1, 2, -3, 1],
        Delay::new(0)
            .cascade(Delay::new(-1))
            .transduce([1, 2, -3, 1, 2, -3])
            .as_slice()
    );
    assert_eq!(
        &['a', 'b', 'c', 'd', 'e', 'f', 'g'],
        Delay::new('b')
            .cascade(Delay::new('a'))
            .transduce(['c', 'd', 'e', 'f', 'g', 'i', 'j'])
            .as_slice()
    );
}

#[test]
fn test_parallel_x() {
    assert_eq!(
        &[2, 1, 3, 1, 1, 4, 1, 1, 1, 5],
        Incr::<1>
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );

    assert_eq!(
        &[
            (2, 3),
            (1, 2),
            (3, 4),
            (1, 2),
            (1, 2),
            (4, 5),
            (1, 2),
            (1, 2),
            (1, 2),
            (5, 6)
        ],
        Incr::<1>
            .parallel(Incr::<2>)
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
    assert_eq!(
        &[5, 3, 7, 3, 3, 9, 3, 3, 3, 11],
        Incr::<1>
            .parallel(Incr::<2>)
            .cascade(Adder::default())
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
}

#[test]
fn test_feedback() {
    assert_eq!(
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        Incr::<1>
            .cascade(Delay::new(0))
            .feedback()
            .transduce(std::iter::repeat(0).take(10))
            .as_slice()
    );
}

#[test]
fn test_adder() {
    assert_eq!(
        &[4, 2, 0, -1],
        Adder::default()
            .transduce([(1, 3), (0, 2), (0, 0), (3, -4)])
            .as_slice()
    );
}

#[test]
fn test_fibo() {
    assert_eq!(
        &[1, 2, 3, 5, 8, 13, 21, 34, 55, 89],
        Delay::new(1)
            .parallel(Delay::new(1).cascade(Delay::default()))
            .cascade(Adder::default())
            .feedback()
            .transduce(std::iter::repeat(0).take(10))
            .as_slice()
    );

    assert_eq!(
        &[1, 2, 3, 5, 8, 13, 21, 34, 55, 89],
        Delay::new(1)
            .parallel(Wire::default())
            .cascade(Adder::default())
            .cascade(Delay::new(1))
            .feedback()
            .transduce(std::iter::repeat(0).take(10))
            .as_slice()
    );
}

#[test]
fn test_double() {
    assert_eq!(
        &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
        Constant::<2>::default()
            .parallel(Wire::default())
            .cascade(Multiplier::default())
            .cascade(Delay::new(1))
            .feedback()
            .transduce(std::iter::repeat(0).take(11))
            .as_slice()
    );

    assert_eq!(
        &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
        Constant::<2>::default()
            .cascade(Multiplier::default().cascade(Delay::new(1)).feedback2())
            .transduce(std::iter::repeat(0).take(11))
            .as_slice()
    );
}

#[test]
fn test_feedback_op() {
    assert_eq!(
        &[0, 0, 1, 3, 6, 10, 15, 21, 28, 36],
        Delay::new(0)
            .feedbackOp(Gain::<i32>::default(), |i1, i2| i1 + i2)
            .transduce(0..10)
            .as_slice()
    );

    let fac = Delay::new(1).feedbackOp(Wire::default(), |i1, i2| i1 * i2);
    // let counter = Incr::<1>.cascade(Delay::new(1)).feedback();
    let counter = Delay::new(1).feedbackOp(Wire::default(), |i1, i2| i1 + i2);
    assert_eq!(
        &[1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800],
        counter
            .cascade(fac)
            .transduce(std::iter::repeat(1).take(11))
            .as_slice()
    );
}
