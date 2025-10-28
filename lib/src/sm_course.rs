use std::ops::Mul;

use crate::sm::StateMachine;

struct Delay<I>(I);
impl<I: Clone> StateMachine<I, I> for Delay<I> {
    type State = Option<I>;

    fn start_state(&self) -> Self::State {
        Some(self.0.clone())
    }

    fn next_values(&self, state: Self::State, input: Option<I>) -> (Self::State, Option<I>) {
        (input, state)
    }
}

pub fn delay<I: Clone>(val0: I) -> impl StateMachine<I, I> {
    Delay(val0)
}

pub fn wire<I>() -> impl StateMachine<I, I> {
    |input| input
}

pub fn scale<I: Mul<I, Output = I> + Clone + 'static>(gain: I) -> impl StateMachine<I, I> {
    move |input| gain.clone() * input
}

#[cfg(test)]
mod tests {
    use crate::{
        sig::IterSignal,
        sm::{StateFullMachine, StateMachine},
        sm_course::{delay, wire},
    };
    use std::ops::Add;

    fn make_incr<const I: i32>() -> impl StateMachine<i32, i32> {
        |input| input + I
    }

    fn make_acc() -> impl StateMachine<i32, i32> {
        (
            |state: i32, input: i32| {
                let acc = state + input;
                (acc, acc)
            },
            0,
        )
    }

    #[test]
    fn it_works() {
        struct Delay2<I>(I, I);
        impl<I: Clone> StateMachine<I, I> for Delay2<I> {
            type State = (Option<I>, Option<I>);

            fn start_state(&self) -> Self::State {
                (Some(self.0.clone()), Some(self.1.clone()))
            }

            fn next_values(
                &self,
                (out, val0): Self::State,
                val1: Option<I>,
            ) -> (Self::State, Option<I>) {
                ((val0, val1), out)
            }
        }

        fn make_delay_2<I: Clone>(val0: I, val1: I) -> impl StateMachine<I, I> {
            Delay2(val0, val1)
        }

        assert_eq!(
            &[100, 10, 1, 0, 2, 0, 0, 3, 0, 0],
            make_delay_2(100, 10)
                .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[10, 100, 0, 0, 0, 0, 0],
            make_delay_2(10, 100)
                .transduce([0, 0, 0, 0, 0, 0, 1])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[-1, 0, 1, 2, -3, 1],
            make_delay_2(-1, 0)
                .transduce([1, 2, -3, 1, 2, -3])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &['a', 'b', 'c', 'd', 'e', 'f', 'g'],
            make_delay_2('a', 'b')
                .transduce(['c', 'd', 'e', 'f', 'g', 'i', 'j'])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_accumulator() {
        assert_eq!(
            &[1, 1, 3, 3, 3, 6, 6, 6, 6, 10],
            make_acc()
                .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    //     //     #[test]
    //     //     fn test_abc() {
    //     //         #[derive(Clone)]
    //     //         enum AbcState {
    //     //             ReadA,
    //     //             ReadB,
    //     //             ReadC,
    //     //             Stop,
    //     //         }

    //     //         fn make_abc() -> impl StateMachine<char, bool> {
    //     //             (
    //     //                 |state, input| {
    //     //                     let (next_state, condition) = match state {
    //     //                         AbcState::ReadA => (AbcState::ReadB, input == 'a'),
    //     //                         AbcState::ReadB => (AbcState::ReadC, input == 'b'),
    //     //                         AbcState::ReadC => (AbcState::ReadA, input == 'c'),
    //     //                         AbcState::Stop => (AbcState::Stop, false),
    //     //                     };
    //     //                     (
    //     //                         if condition {
    //     //                             next_state
    //     //                         } else {
    //     //                             AbcState::Stop
    //     //                         },
    //     //                         condition,
    //     //                     )
    //     //                 },
    //     //                 AbcState::ReadA,
    //     //             )
    //     //                 .into_state_machine()
    //     //         }

    //     //         assert_eq!(
    //     //             &[true, false, false],
    //     //             make_abc()
    //     //                 .transduce(['a', 'a', 'a'])
    //     //                 .collect::<Vec<_>>()
    //     //                 .as_slice()
    //     //         );
    //     //         assert_eq!(
    //     //             &[true, true, true, true, false, false, false],
    //     //             make_abc()
    //     //                 .transduce(['a', 'b', 'c', 'a', 'c', 'b', 'a'])
    //     //                 .collect::<Vec<_>>()
    //     //                 .as_slice()
    //     //         );
    //     //     }

    #[test]
    fn test_average2() {
        fn make_avg() -> impl StateMachine<i32, f32> {
            (
                |state, input| {
                    let input = input as f32;
                    let out = (input + state) / 2.0;
                    (input, out)
                },
                0.0,
            )
        }
        assert_eq!(
            &[5.0, 7.5, 3.5, 6.0],
            make_avg()
                .transduce([10, 5, 2, 10])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_cascade() {
        assert_eq!(
            &[100, 10, 1, 0, 2, 0, 0, 3, 0, 0],
            delay(10)
                .cascade(delay(100))
                .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[10, 100, 0, 0, 0, 0, 0],
            delay(100)
                .cascade(delay(10))
                .transduce([0, 0, 0, 0, 0, 0, 1])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[-1, 0, 1, 2, -3, 1],
            delay(0)
                .cascade(delay(-1))
                .transduce([1, 2, -3, 1, 2, -3])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &['a', 'b', 'c', 'd', 'e', 'f', 'g'],
            delay('b')
                .cascade(delay('a'))
                .transduce(['c', 'd', 'e', 'f', 'g', 'i', 'j'])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_parallel() {
        assert_eq!(
            &[2, 1, 3, 1, 1, 4, 1, 1, 1, 5],
            make_incr::<1>()
                .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
                .collect::<Vec<_>>()
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
            make_incr::<1>()
                .parallel(make_incr::<2>())
                .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[5, 3, 7, 3, 3, 9, 3, 3, 3, 11],
            make_incr::<1>()
                .parallel(make_incr::<2>())
                .cascade(|(i1, i2)| i1 + i2)
                .transduce([1, 0, 2, 0, 0, 3, 0, 0, 0, 4])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_feedback() {
        assert_eq!(
            &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
            make_incr::<1>()
                .cascade(delay(0))
                .feedback()
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_adder() {
        assert_eq!(
            &[4, 2, 0, -1],
            (|(i1, i2)| i1 + i2)
                .transduce([(1, 3), (0, 2), (0, 0), (3, -4)])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_fibo() {
        assert_eq!(
            &[1, 2, 3, 5, 8, 13, 21, 34, 55, 89],
            delay(1)
                .parallel(delay(1).cascade(delay(0)))
                .cascade(|(i1, i2)| i1 + i2)
                .feedback()
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );

        assert_eq!(
            &[1, 2, 3, 5, 8, 13, 21, 34, 55, 89],
            delay(1)
                .parallel(|i| i * 1)
                .cascade(|(i1, i2)| i1 + i2)
                .cascade(delay(1))
                .feedback()
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_double() {
        assert_eq!(
            &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
            ((|_| 2)
                .parallel(|i| i * 1)
                .cascade(|(i1, i2)| i1 * i2)
                .cascade(delay(1))
                .feedback())
            .transduce(0..11)
            .collect::<Vec<_>>()
            .as_slice()
        );

        assert_eq!(
            &[1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024],
            (|_i: i32| 2)
                .cascade((|(i1, i2)| i1 * i2).cascade(delay(1)).feedback2())
                .transduce(0..11)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_feedback_op() {
        // acc
        assert_eq!(
            &[0, 0, 1, 3, 6, 10, 15, 21, 28, 36],
            delay(0)
                .feedback_op(|i| i * 1, |i1: i32, i2| i1 + i2)
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );

        let fac = delay(1).feedback_op(|i| i * 1, |i1, i2| i1 * i2);
        let counter = delay(1).feedback_op(|i| i * 1, |i1: i32, i2| i1 + i2);
        assert_eq!(
            &[1, 1, 2, 6, 24, 120, 720, 5040, 40320, 362880, 3628800],
            counter
                .cascade(fac)
                .transduce(std::iter::repeat(1).take(11))
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_switch() {
        assert_eq!(
            &[0, 3, 4, 9, 8, 15, 12, 21, 16, 27],
            (|i: i32| i * 2)
                .switch(|i| i * 3, |i| i % 2 == 0)
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_mux() {
        assert_eq!(
            &[0, 3, 4, 9, 8, 15, 12, 21, 16, 27],
            (|i: i32| i * 2)
                .mux(|i| i * 3, |i| i % 2 == 0)
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_mux_vs_switch() {
        assert_eq!(
            &[2, 5, 9, 200, 500, 900, 10, 12, 15],
            make_acc()
                .switch(make_acc(), |i| i > 100)
                .transduce([2, 3, 4, 200, 300, 400, 1, 2, 3])
                .collect::<Vec<_>>()
                .as_slice()
        );

        assert_eq!(
            &[2, 5, 9, 209, 509, 909, 910, 912, 915],
            make_acc()
                .mux(make_acc(), |i| i > 100)
                .transduce([2, 3, 4, 200, 300, 400, 1, 2, 3])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_if() {
        assert_eq!(
            &[0, 2, 4, 6, 8, 10, 12, 14, 16, 18],
            (|i: i32| i * 2)
                .r#if(|i| i * 3, |i| i % 2 == 0)
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[0, 3, 6, 9, 12, 15, 18, 21, 24, 27],
            (|i: i32| i * 2)
                .r#if(|i| i * 3, |i| i % 2 != 0)
                .transduce(0..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    fn make_consume_five() -> impl StateMachine<i32, Option<i32>> {
        (
            |state: (i32, i32), input: i32| match state.0 {
                s if s < 4 => ((state.0 + 1, state.1 + input), None),
                4 => {
                    let out = state.1 + input;
                    ((state.0 + 1, out), Some(out))
                }
                _ => ((state.0, state.1), Some(state.1)),
            },
            |state: (i32, i32)| state.0 == 5,
            (0, 0),
        )
    }

    #[test]
    fn test_consume_five() {
        let mut sm = make_consume_five().into_state_full_machine();
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
            make_consume_five()
                .transduce(1..10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    fn make_char_tsm(c: char) -> impl StateMachine<char, char> {
        (move |_state, _input| (true, c), |state| state, false)
    }

    #[test]
    fn test_repeat() {
        assert_eq!(
            &['a'],
            make_char_tsm('a')
                .transduce(0..1)
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &['a', 'a', 'a', 'a'],
            make_char_tsm('a')
                .repeat(Some(4))
                .run()
                .collect::<Vec<_>>()
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
            make_consume_five()
                .repeat_until(|i| i > 10)
                .transduce(0..20)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_until() {
        assert_eq!(
            &[None, None, None, None, Some(10),],
            make_consume_five()
                .until(|i| i > 10)
                .transduce(0..20)
                .collect::<Vec<_>>()
                .as_slice()
        );
        assert_eq!(
            &[None, None, None],
            make_consume_five()
                .until(|i| i == 2)
                .transduce(0..20)
                .collect::<Vec<_>>()
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
            make_consume_five()
                .repeat(None)
                .until(|i| i > 10)
                .transduce(0..20)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_seq() {
        assert_eq!(
            &['a', 'b', 'c'],
            make_char_tsm('a')
                .seq(make_char_tsm('b'))
                .seq(make_char_tsm('c'))
                .run()
                .collect::<Vec<_>>()
                .as_slice()
        );

        assert_eq!(
            &['a', 'b', 'c'],
            crate::seq!(make_char_tsm('a'), make_char_tsm('b'), make_char_tsm('c'))
                .run()
                .collect::<Vec<_>>()
                .as_slice()
        );

        macro_rules! make_text_sequence {
                    ($first:expr $(, $rest:expr)+ $(,)?) => {{
                        $crate::seq!(
                        make_char_tsm($first)
                            $(
                                ,make_char_tsm($rest)
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
                .run()
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn exos() {
        fn make_my_sm() -> impl StateMachine<i32, i32> {
            (
                |state, input| {
                    let (count, acc) = state;
                    let out = acc + input;
                    if out >= 100 {
                        ((count + 1, 0), out)
                    } else {
                        ((count, out), out)
                    }
                },
                |state: (i32, i32)| state.0 >= 3,
                (0, 0),
            )
        }
        assert_eq!(
            [1, 3, 6, 106, 4, 13, 513, 51, 49, 106],
            make_my_sm()
                .transduce([
                    1, 2, 3, 100, 4, 9, 500, 51, -2, 57, 103, 1, 1, 1, 1, -10, 207, 3, 1
                ])
                .collect::<Vec<_>>()
                .as_slice()
        );

        fn make_my_new_sm() -> impl StateMachine<i32, i32> {
            (
                |state, input| {
                    let out = state + input;
                    (out, out)
                },
                |state| state > 100,
                0,
            )
        }
        assert_eq!(
            [1, 3, 6, 106, 4, 13, 513, 51, 49, 106],
            make_my_new_sm()
                .repeat(Some(3))
                .transduce([
                    1, 2, 3, 100, 4, 9, 500, 51, -2, 57, 103, 1, 1, 1, 1, -10, 207, 3, 1
                ])
                .collect::<Vec<_>>()
                .as_slice()
        );
    }

    #[test]
    fn test_transduce_signal() {
        let acc = (wire()).feedback_op(delay(0), i32::add);
        assert_eq!(
            &[1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
            IterSignal::new(acc.transduce_signal(|n| if n == 0 { 1 } else { 0 }))
                .take(10)
                .collect::<Vec<_>>()
                .as_slice()
        );
    }
}
