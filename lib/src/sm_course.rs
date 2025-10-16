use crate::sm::StateFullMachine;
use crate::sm::StateMachine;
use std::{marker::PhantomData, ops::Add};

pub struct Incr<const STEP: i64>;
impl<const STEP: i64> StateMachine<i64> for Incr<STEP> {
    type State = ();
    type Output = i64;

    fn next_values(
        &self,
        _state: Self::State,
        input: Option<i64>,
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

impl<I> StateMachine<I> for Delay<I>
where
    I: Clone,
{
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
struct Acc<I> {
    _phantom: PhantomData<I>,
}

impl<I> StateMachine<I> for Acc<I>
where
    I: Clone + Default,
    I: Add<I, Output = I>,
{
    type State = I;
    type Output = I;

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        if let Some(input) = input {
            let acc = state + input;
            let out = Some(acc.clone());
            (acc, out)
        } else {
            (state, None)
        }
    }

    fn start_state(&self) -> Self::State {
        I::default()
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

    impl<I> StateMachine<I> for Delay2<I>
    where
        I: Clone,
    {
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
            .into_state_full()
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
    assert_eq!(
        &[10, 100, 0, 0, 0, 0, 0],
        Delay2::new(10, 100)
            .into_state_full()
            .transduce([0, 0, 0, 0, 0, 0, 1])
            .as_slice()
    );
    assert_eq!(
        &[-1, 0, 1, 2, -3, 1],
        Delay2::new(-1, 0)
            .into_state_full()
            .transduce([1, 2, -3, 1, 2, -3])
            .as_slice()
    );
    assert_eq!(
        &['a', 'b', 'c', 'd', 'e', 'f', 'g'],
        Delay2::new('a', 'b')
            .into_state_full()
            .transduce(['c', 'd', 'e', 'f', 'g', 'i', 'j'])
            .as_slice()
    );
}

#[test]
fn test_accumulator() {
    assert_eq!(
        &[1, 1, 3, 3, 3, 6, 6, 6, 6, 10],
        Acc::default()
            .into_state_full()
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
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
        Abc.into_state_full().transduce(['a', 'a', 'a']).as_slice()
    );
    assert_eq!(
        &[true, true, true, true, false, false, false],
        Abc.into_state_full()
            .transduce(['a', 'b', 'c', 'a', 'c', 'b', 'a'])
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
        Average2
            .into_state_full()
            .transduce([10, 5, 2, 10])
            .as_slice()
    );
}

#[test]
fn test_cascade() {
    assert_eq!(
        &[100, 10, 1, 0, 2, 0, 0, 3, 0, 0],
        Delay::new(10)
            .cascade(Delay::new(100))
            .into_state_full()
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
    assert_eq!(
        &[10, 100, 0, 0, 0, 0, 0],
        Delay::new(100)
            .cascade(Delay::new(10))
            .into_state_full()
            .transduce([0, 0, 0, 0, 0, 0, 1])
            .as_slice()
    );
    assert_eq!(
        &[-1, 0, 1, 2, -3, 1],
        Delay::new(0)
            .cascade(Delay::new(-1))
            .into_state_full()
            .transduce([1, 2, -3, 1, 2, -3])
            .as_slice()
    );
    assert_eq!(
        &['a', 'b', 'c', 'd', 'e', 'f', 'g'],
        Delay::new('b')
            .cascade(Delay::new('a'))
            .into_state_full()
            .transduce(['c', 'd', 'e', 'f', 'g', 'i', 'j'])
            .as_slice()
    );
}

#[test]
fn test_parallel_x() {
    assert_eq!(
        &[2, 1, 3, 1, 1, 4, 1, 1, 1, 5],
        Incr::<1>
            .into_state_full()
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
            .into_state_full()
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
    assert_eq!(
        &[5, 3, 7, 3, 3, 9, 3, 3, 3, 11],
        Incr::<1>
            .parallel(Incr::<2>)
            .cascade(|(i1, i2)| i1 + i2)
            .into_state_full()
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
            .into_state_full()
            .run(Some(10))
            .as_slice()
    );
}

#[test]
fn test_adder() {
    assert_eq!(
        &[4, 2, 0, -1],
        (|(i1, i2)| i1 + i2)
            .into_state_full()
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
            .cascade(|(i1, i2)| i1 + i2)
            .feedback()
            .into_state_full()
            .run(Some(10))
            .as_slice()
    );

    assert_eq!(
        &[1, 2, 3, 5, 8, 13, 21, 34, 55, 89],
        Delay::new(1)
            .parallel(|i| i * 1)
            .cascade(|(i1, i2)| i1 + i2)
            .cascade(Delay::new(1))
            .feedback()
            .into_state_full()
            .run(Some(10))
            .as_slice()
    );
}

#[test]
fn test_double() {
    assert_eq!(
        &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
        (|_| 2)
            .parallel(|i| i * 1)
            .cascade(|(i1, i2)| i1 * i2)
            .cascade(Delay::new(1))
            .feedback()
            .into_state_full()
            .run(Some(11))
            .as_slice()
    );

    assert_eq!(
        &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
        (|_i: i32| 2)
            .cascade((|(i1, i2)| i1 * i2).cascade(Delay::new(1)).feedback2())
            .into_state_full()
            .run(Some(11))
            .as_slice()
    );
}

#[test]
fn test_feedback_op() {
    assert_eq!(
        &[0, 0, 1, 3, 6, 10, 15, 21, 28, 36],
        Delay::new(0)
            .feedback_op(|i| i * 1, |i1, i2| i1 + i2)
            .into_state_full()
            .transduce(0..10)
            .as_slice()
    );

    let fac = Delay::new(1).feedback_op(|i| i * 1, |i1, i2| i1 * i2);
    // let counter = Incr::<1>.cascade(Delay::new(1)).feedback();
    let counter = Delay::new(1).feedback_op(|i| i * 1, |i1, i2| i1 + i2);
    assert_eq!(
        &[1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800],
        counter
            .cascade(fac)
            .into_state_full()
            .transduce(std::iter::repeat(1).take(11))
            .as_slice()
    );
}

#[test]
fn test_switch() {
    assert_eq!(
        &[0, 3, 4, 9, 8, 15, 12, 21, 16, 27],
        (|i| i * 2)
            .switch(|i| i * 3, |i| i % 2 == 0)
            .into_state_full()
            .transduce(0..10)
            .as_slice()
    );
}

#[test]
fn test_mux() {
    assert_eq!(
        &[0, 3, 4, 9, 8, 15, 12, 21, 16, 27],
        (|i| i * 2)
            .mux(|i| i * 3, |i| i % 2 == 0)
            .into_state_full()
            .transduce(0..10)
            .as_slice()
    );
}

#[test]
fn test_mux_vs_switch() {
    assert_eq!(
        &[2, 5, 9, 200, 500, 900, 10, 12, 15],
        Acc::default()
            .switch(Acc::default(), |i| i > 100)
            .into_state_full()
            .transduce([2, 3, 4, 200, 300, 400, 1, 2, 3])
            .as_slice()
    );

    assert_eq!(
        &[2, 5, 9, 209, 509, 909, 910, 912, 915],
        Acc::default()
            .mux(Acc::default(), |i| i > 100)
            .into_state_full()
            .transduce([2, 3, 4, 200, 300, 400, 1, 2, 3])
            .as_slice()
    );
}

#[test]
fn test_if() {
    assert_eq!(
        &[0, 2, 4, 6, 8, 10, 12, 14, 16, 18],
        (|i| i * 2)
            .r#if(|i| i * 3, |i| i % 2 == 0)
            .into_state_full()
            .transduce(0..10)
            .as_slice()
    );
    assert_eq!(
        &[0, 3, 6, 9, 12, 15, 18, 21, 24, 27],
        (|i| i * 2)
            .r#if(|i| i * 3, |i| i % 2 != 0)
            .into_state_full()
            .transduce(0..10)
            .as_slice()
    );
}

struct ConsumeFive;
impl StateMachine<i32> for ConsumeFive {
    type State = (i32, i32);
    type Output = Option<i32>;

    fn start_state(&self) -> Self::State {
        (0, 0)
    }

    fn done(&self, state: Self::State) -> bool {
        state.0 == 5
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<i32>,
    ) -> (Self::State, Option<Self::Output>) {
        let input = input.unwrap();
        match state.0 {
            s if s < 4 => ((state.0 + 1, state.1 + input), Some(None)),
            4 => {
                let out = state.1 + input;
                ((state.0 + 1, out), Some(Some(out)))
            }
            _ => ((state.0, state.1), Some(Some(state.1))),
        }
    }
}
#[test]
fn test_consume_five() {
    let mut sm = ConsumeFive.into_state_full();
    assert_eq!(sm.step(Some(1)), Some(None));
    assert_eq!(sm.is_done(), false);

    assert_eq!(sm.step(Some(1)), Some(None));
    assert_eq!(sm.is_done(), false);

    assert_eq!(sm.step(Some(1)), Some(None));
    assert_eq!(sm.is_done(), false);

    assert_eq!(sm.step(Some(1)), Some(None));
    assert_eq!(sm.is_done(), false);

    assert_eq!(sm.step(Some(1)), Some(Some(5)));
    assert_eq!(sm.is_done(), true);

    assert_eq!(sm.step(Some(1)), Some(Some(5)));
    assert_eq!(sm.is_done(), true);

    assert_eq!(
        &[None, None, None, None, Some(15)],
        ConsumeFive.into_state_full().transduce(1..10).as_slice()
    );
}

struct CharTSM(char);

impl StateMachine<char> for CharTSM {
    type State = bool;
    type Output = char;

    fn start_state(&self) -> Self::State {
        false
    }

    fn done(&self, state: Self::State) -> bool {
        state
    }

    fn next_values(
        &self,
        _state: Self::State,
        _input: Option<char>,
    ) -> (Self::State, Option<Self::Output>) {
        (true, Some(self.0))
    }
}

#[test]
fn test_repeat() {
    assert_eq!(&['a'], CharTSM('a').into_state_full().run(None).as_slice());
    assert_eq!(
        &['a', 'a', 'a', 'a'],
        CharTSM('a')
            .repeat(Some(4))
            .into_state_full()
            .run(Some(10))
            .as_slice()
    )
}

#[test]
fn test_repeat_until() {
    assert_eq!(
        &[
            None,
            None,
            None,
            None,
            Some(10),
            None,
            None,
            None,
            None,
            Some(35),
            None,
            None,
            None,
            None,
            Some(60)
        ],
        ConsumeFive
            .repeat_until(|i| i > 10)
            .into_state_full()
            .transduce(0..20)
            .as_slice()
    );
}

#[test]
fn test_until() {
    assert_eq!(
        &[None, None, None, None, Some(10),],
        ConsumeFive
            .until(|i| i > 10)
            .into_state_full()
            .transduce(0..20)
            .as_slice()
    );
    assert_eq!(
        &[None, None, None],
        ConsumeFive
            .until(|i| i == 2)
            .into_state_full()
            .transduce(0..20)
            .as_slice()
    );
    assert_eq!(
        [
            None,
            None,
            None,
            None,
            Some(10),
            None,
            None,
            None,
            None,
            Some(35),
            None,
            None
        ],
        ConsumeFive
            .repeat(None)
            .until(|i| i > 10)
            .into_state_full()
            .transduce(0..20)
            .as_slice()
    );
}

#[test]
fn test_seq() {
    assert_eq!(
        &['a', 'b', 'c'],
        CharTSM('a')
            .seq(CharTSM('b'))
            .seq(CharTSM('c'))
            .into_state_full()
            .run(None)
            .as_slice()
    );

    assert_eq!(
        &['a', 'b', 'c'],
        crate::seq!(CharTSM('a'), CharTSM('b'), CharTSM('c'))
            .into_state_full()
            .run(None)
            .as_slice()
    );

    macro_rules! make_text_sequence {
        ($first:expr $(, $rest:expr)+ $(,)?) => {{
            $crate::seq!(
            CharTSM($first)
                $(
                    ,CharTSM($rest)
                )+
            )
        }};
    }
    assert_eq!(
        [
            'h', 'e', 'l', 'l', 'o', 'h', 'e', 'l', 'l', 'o', 'h', 'e', 'l', 'l', 'o'
        ],
        make_text_sequence!('h', 'e', 'l', 'l', 'o')
            .repeat(Some(3))
            .into_state_full()
            .run(None)
            .as_slice()
    );
}

#[test]
fn exos() {
    struct MySM;
    impl StateMachine<i32> for MySM {
        type State = (i32, i32);
        type Output = i32;

        fn start_state(&self) -> Self::State {
            (0, 0)
        }

        fn done(&self, state: Self::State) -> bool {
            state.0 >= 3
        }

        fn next_values(
            &self,
            state: Self::State,
            input: Option<i32>,
        ) -> (Self::State, Option<Self::Output>) {
            let input = input.expect("no input");
            let (count, acc) = state;
            let out = acc + input;
            if out >= 100 {
                ((count + 1, 0), Some(out))
            } else {
                ((count, out), Some(out))
            }
        }
    }
    assert_eq!(
        [1, 3, 6, 106, 4, 13, 513, 51, 49, 106],
        MySM.into_state_full()
            .transduce([
                1, 2, 3, 100, 4, 9, 500, 51, -2, 57, 103, 1, 1, 1, 1, -10, 207, 3, 1
            ])
            .as_slice()
    );
    struct MyNewSM;
    impl StateMachine<i32> for MyNewSM {
        type State = i32;
        type Output = i32;

        fn start_state(&self) -> Self::State {
            0
        }

        fn done(&self, state: Self::State) -> bool {
            state > 100
        }

        fn next_values(
            &self,
            state: Self::State,
            input: Option<i32>,
        ) -> (Self::State, Option<Self::Output>) {
            let out = state + input.expect("no input");
            (out, Some(out))
        }
    }
    assert_eq!(
        [1, 3, 6, 106, 4, 13, 513, 51, 49, 106],
        MyNewSM
            .repeat(Some(3))
            .into_state_full()
            .transduce([
                1, 2, 3, 100, 4, 9, 500, 51, -2, 57, 103, 1, 1, 1, 1, -10, 207, 3, 1
            ])
            .as_slice()
    );
}
