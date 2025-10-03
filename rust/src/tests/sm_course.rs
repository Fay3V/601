use crate::sm::StateMachine;
use core::fmt;
struct Incr<const STEP: i32>;

impl<const STEP: i32> StateMachine<i32> for Incr<STEP> {
    type State = ();
    type Output = i32;

    fn next_values(&self, _state: Self::State, input: i32) -> (Self::State, Self::Output) {
        ((), input + STEP)
    }

    fn start_state(&self) -> Self::State {
        ()
    }
}

struct Delay<I> {
    val: I,
}

impl<I> Delay<I> {
    fn new(val: I) -> Self {
        Self { val }
    }
}

impl<I: Clone + fmt::Debug> StateMachine<I> for Delay<I> {
    type State = I;
    type Output = I;

    fn next_values(&self, state: Self::State, input: I) -> (Self::State, Self::Output) {
        (input, state)
    }

    fn start_state(&self) -> Self::State {
        self.val.clone()
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
        type State = (I, I);
        type Output = I;

        fn next_values(&self, state: Self::State, input: I) -> (Self::State, Self::Output) {
            ((state.1, input), state.0)
        }

        fn start_state(&self) -> Self::State {
            (self.val0.clone(), self.val1.clone())
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

        fn next_values(&self, state: Self::State, input: i32) -> (Self::State, Self::Output) {
            let acc = state + input;
            (acc, acc)
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

        fn next_values(&self, state: Self::State, input: char) -> (Self::State, Self::Output) {
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
                condition,
            )
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
        type State = i32;
        type Output = f32;

        fn next_values(&self, state: Self::State, input: i32) -> (Self::State, Self::Output) {
            (input, (state as f32 + input as f32) / 2.0)
        }

        fn start_state(&self) -> Self::State {
            0
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
            .parallel_add(Incr::<2>)
            .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
            .as_slice()
    );
}
