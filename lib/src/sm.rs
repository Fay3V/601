use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Rigth(R),
}

pub trait StateFullMachine<In, Out> {
    fn reset(&mut self);
    fn step(&mut self, input: Option<In>) -> Option<Out>;
    fn is_done(&self) -> bool;
}

pub struct StateFull<In, Out, SM>(SM::State, SM)
where
    SM: StateMachine<In, Out>;

impl<In, Out, SM> StateFull<In, Out, SM>
where
    SM: StateMachine<In, Out>,
{
    pub fn new(sm: SM) -> Self {
        Self(sm.start_state(), sm)
    }
}
impl<In, Out, SM> StateFullMachine<In, Out> for StateFull<In, Out, SM>
where
    SM: StateMachine<In, Out>,
    SM::State: Clone,
{
    fn reset(&mut self) {
        self.0 = self.1.start_state();
    }

    fn step(&mut self, input: Option<In>) -> Option<Out> {
        let (new_state, output) = self.1.next_values(self.0.clone(), input);
        self.0 = new_state;
        output
    }

    fn is_done(&self) -> bool {
        self.1.done(self.0.clone())
    }
}

pub trait StateMachine<In, Out> {
    type State: Clone;

    fn start_state(&self) -> Self::State;
    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>);

    fn done(&self, _state: Self::State) -> bool {
        false
    }

    fn transduce<I, II: IntoIterator<Item = I>>(&self, inputs: II) -> impl Iterator<Item = Out>
    where
        Self: Sized,
        I: Into<In>,
    {
        let mut state = self.start_state();
        inputs.into_iter().map_while(move |input| {
            if self.done(state.clone()) {
                None
            } else {
                let (new_state, output) = self.next_values(state.clone(), Some(input.into()));
                state = new_state;
                output
            }
        })
        // .collect()
    }

    fn run(&self) -> impl Iterator<Item = Out>
    where
        In: Default,
        Self: Sized,
    {
        let mut state = self.start_state();
        (0..).into_iter().map_while(move |_input| {
            if self.done(state.clone()) {
                None
            } else {
                let (new_state, output) = self.next_values(state.clone(), Some(In::default()));
                state = new_state;
                output
            }
        })
    }

    fn into_state_full_machine(self) -> impl StateFullMachine<In, Out>
    where
        Self: Sized,
    {
        StateFull::new(self)
    }

    fn cascade<O, SM>(self, sm: SM) -> Cascade<Self, SM, O>
    where
        Self: Sized,
    {
        Cascade {
            first_machine: self,
            second_machine: sm,
            _phantom: PhantomData,
        }
    }

    fn parallel<SM, O>(self, next_machine: SM) -> Parallel<Self, SM>
    where
        In: Clone,
        Self: Sized,
        SM: StateMachine<In, O>,
    {
        Parallel {
            machine1: self,
            machine2: next_machine,
        }
    }

    // //     fn parallel2<I2, SM2>(self, next_machine: SM2) -> Parallel2<Self, SM2>
    // //     where
    // //         Self: Sized,
    // //         I2: Clone,
    // //         SM2: StateMachine<I2>,
    // //     {
    // //         Parallel2 {
    // //             machine1: self,
    // //             machine2: next_machine,
    // //         }
    // //     }

    fn feedback(self) -> Feedback<Self>
    where
        In: Clone,
        Self: Sized,
    {
        Feedback { machine: self }
    }

    fn feedback2(self) -> Feedback2<Self>
    where
        Self: Sized,
        Out: Clone,
    {
        Feedback2 { machine: self }
    }

    // //     fn feedback_op<SM, Op>(self, machine: SM, op: Op) -> FeedbackOp<Self, SM, Op, Input>
    // //     where
    // //         Self: Sized,
    // //     {
    // //         FeedbackOp {
    // //             first_machine: self,
    // //             second_machine: machine,
    // //             op,
    // //             _phantom: PhantomData,
    // //         }
    // //     }

    fn switch<SM, P>(self, machine: SM, pred: P) -> Switch<Self, SM, P>
    where
        Self: Sized,
        SM: StateMachine<In, Out>,
        In: Clone,
        P: Fn(In) -> bool,
    {
        Switch {
            first_machine: self,
            second_machine: machine,
            pred,
        }
    }

    fn mux<SM, P>(self, machine: SM, pred: P) -> Mux<Self, SM, P>
    where
        Self: Sized,
        SM: StateMachine<In, Out>,
        In: Clone,
        P: Fn(In) -> bool,
    {
        Mux {
            first_machine: self,
            second_machine: machine,
            pred,
        }
    }

    fn r#if<SM, P>(self, machine: SM, pred: P) -> If<Self, SM, P>
    where
        Self: Sized,
        SM: StateMachine<In, Out>,
        In: Clone,
        P: Fn(In) -> bool,
    {
        If {
            first_machine: self,
            second_machine: machine,
            pred,
        }
    }

    fn seq<SM>(self, machine: SM) -> Seq<Self, SM>
    where
        Self: Sized,
        SM: StateMachine<In, Out>,
    {
        Seq {
            first_machine: self,
            second_machine: machine,
        }
    }

    fn repeat(self, n: Option<usize>) -> Repeat<Self>
    where
        Self: Sized,
    {
        Repeat { machine: self, n }
    }

    fn repeat_until<P>(self, pred: P) -> RepeatUntil<Self, P>
    where
        Self: Sized,
        In: Clone,
        P: Fn(In) -> bool,
    {
        RepeatUntil {
            machine: self,
            pred,
        }
    }

    fn until<P>(self, pred: P) -> Until<Self, P>
    where
        Self: Sized,
        In: Clone,
        P: Fn(In) -> bool,
    {
        Until {
            machine: self,
            pred,
        }
    }
}

// pub trait StateMachineExt {
//     fn cascade<In, Out, O, SM>(self, sm: SM) -> impl IntoStateMachine<In, O>
//     where
//         Self: Sized,
//         SM: IntoStateMachine<Out, O>,
//         Self: IntoStateMachine<In, Out>;

//     fn parallel<In, Out, SM, O>(self, next_machine: SM) -> impl IntoStateMachine<In, (Out, O)>
//     where
//         In: Clone,
//         Self: Sized,
//         SM: IntoStateMachine<In, O>,
//         Self: IntoStateMachine<In, Out>;

//     // //     fn parallel2<I2, SM2>(self, next_machine: SM2) -> Parallel2<Self, SM2>
//     // //     where
//     // //         Self: Sized,
//     // //         I2: Clone,
//     // //         SM2: StateMachine<I2>,
//     // //     {
//     // //         Parallel2 {
//     // //             machine1: self,
//     // //             machine2: next_machine,
//     // //         }
//     // //     }

//     fn feedback<In>(self) -> impl IntoStateMachine<In, In>
//     where
//         In: Clone,
//         Self: Sized,
//         Self: IntoStateMachine<In, In>;

//     fn feedback2<In, Out>(self) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<(In, Out), Out>,
//         Out: Clone;

//     // //     fn feedback_op<SM, Op>(self, machine: SM, op: Op) -> FeedbackOp<Self, SM, Op, Input>
//     // //     where
//     // //         Self: Sized,
//     // //     {
//     // //         FeedbackOp {
//     // //             first_machine: self,
//     // //             second_machine: machine,
//     // //             op,
//     // //             _phantom: PhantomData,
//     // //         }
//     // //     }

//     fn switch<In, Out, SM, P>(self, machine: SM, pred: P) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>,
//         SM: IntoStateMachine<In, Out>,
//         In: Clone,
//         P: Fn(In) -> bool;

//     fn mux<In, Out, SM, P>(self, machine: SM, pred: P) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>,
//         SM: IntoStateMachine<In, Out>,
//         In: Clone,
//         P: Fn(In) -> bool;

//     fn r#if<In, Out, SM, P>(self, machine: SM, pred: P) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>,
//         SM: IntoStateMachine<In, Out>,
//         In: Clone,
//         P: Fn(In) -> bool;

//     fn repeat<In, Out>(self, n: Option<usize>) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>;

//     fn repeat_until<In, Out, P>(self, pred: P) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>,
//         In: Clone,
//         P: Fn(In) -> bool;

//     fn until<In, Out, P>(self, pred: P) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>,
//         In: Clone,
//         P: Fn(In) -> bool;

//     fn seq<In, Out, SM>(self, machine: SM) -> impl IntoStateMachine<In, Out>
//     where
//         Self: Sized,
//         Self: IntoStateMachine<In, Out>,
//         SM: IntoStateMachine<In, Out>;
// }

// impl<M> StateMachineExt for M {}

// pub trait IntoStateMachine<In, Out> {
//     fn into_state_machine(self) -> impl StateMachine<In, Out>;

//     fn into_state_full_machine(self) -> impl StateFullMachine<In, Out>
//     where
//         Self: Sized,
//     {
//         StateFull::new(self.into_state_machine())
//     }
// }

// impl<In, Out, SM> IntoStateMachine<In, Out> for SM
// where
//     SM: StateMachine<In, Out>,
// {
//     fn into_state_machine(self) -> impl StateMachine<In, Out> {
//         self
//     }
// }

impl<In, Out, F> StateMachine<In, Out> for F
where
    F: Fn(In) -> Out + 'static,
{
    type State = ();

    fn start_state(&self) -> Self::State {
        ()
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        if let Some(input) = input {
            let out = self(input);
            ((), Some(out))
        } else {
            (state, None)
        }
    }
}

impl<In, Out, S, F> StateMachine<In, Out> for (F, S)
where
    S: Clone,
    F: Fn(S, In) -> (S, Out) + 'static,
{
    type State = S;

    fn start_state(&self) -> Self::State {
        self.1.clone()
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        if let Some(input) = input {
            let (new_state, out) = (self.0)(state, input);
            (new_state, Some(out))
        } else {
            (state, None)
        }
    }
}

impl<In, Out, S, F, D> StateMachine<In, Out> for (F, D, S)
where
    S: Clone,
    F: Fn(S, In) -> (S, Out) + 'static,
    D: Fn(S) -> bool + 'static,
{
    type State = S;

    fn start_state(&self) -> Self::State {
        self.2.clone()
    }

    fn done(&self, state: Self::State) -> bool {
        (self.1)(state)
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        if let Some(input) = input {
            let (new_state, out) = (self.0)(state, input);
            (new_state, Some(out))
        } else {
            (state, None)
        }
    }
}

pub struct Until<SM, P> {
    machine: SM,
    pred: P,
}

impl<In, Out, SM, P> StateMachine<In, Out> for Until<SM, P>
where
    In: Clone,
    SM: StateMachine<In, Out>,
    P: Fn(In) -> bool,
{
    type State = (SM::State, bool);

    fn start_state(&self) -> Self::State {
        (self.machine.start_state(), false)
    }

    fn done(&self, state: Self::State) -> bool {
        let (state, done) = state;
        self.machine.done(state) || done
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (state, _) = state;
        let done = input
            .clone()
            .map(|input| (self.pred)(input))
            .unwrap_or(false);
        let (new_s, out) = self.machine.next_values(state, input.clone());
        ((new_s, done), out)
    }
}

pub struct RepeatUntil<SM, P> {
    machine: SM,
    pred: P,
}

impl<In, Out, SM, P> StateMachine<In, Out> for RepeatUntil<SM, P>
where
    In: Clone,
    SM: StateMachine<In, Out>,
    P: Fn(In) -> bool,
{
    type State = (SM::State, bool);

    fn start_state(&self) -> Self::State {
        (self.machine.start_state(), false)
    }

    fn done(&self, state: Self::State) -> bool {
        let (state, done) = state;
        self.machine.done(state) && done
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (state, _) = state;
        let done = input
            .clone()
            .map(|input| (self.pred)(input))
            .unwrap_or(false);
        let (mut new_s, out) = self.machine.next_values(state, input.clone());
        if self.machine.done(new_s.clone()) && !done {
            new_s = self.machine.start_state();
        }
        ((new_s, done), out)
    }
}

pub struct Repeat<SM> {
    machine: SM,
    n: Option<usize>,
}

impl<In, Out, SM> StateMachine<In, Out> for Repeat<SM>
where
    SM: StateMachine<In, Out>,
    SM::State: Clone,
{
    type State = (usize, SM::State);

    fn start_state(&self) -> Self::State {
        (0, self.machine.start_state())
    }

    fn done(&self, state: Self::State) -> bool {
        self.n == Some(state.0)
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (mut n, state) = state;
        let (mut new_s, out) = self.machine.next_values(state, input);
        if self.machine.done(new_s.clone()) && Some(n) != self.n {
            new_s = self.machine.start_state();
            n += 1;
        }
        ((n, new_s), out)
    }
}

#[macro_export]
macro_rules! seq {
($first:expr $(, $rest:expr)+ $(,)?) => {{
    $first
        $(
            .seq($rest)
        )+
}};
}

pub struct Seq<SM1, SM2> {
    first_machine: SM1,
    second_machine: SM2,
}

impl<In, Out, SM1, SM2> StateMachine<In, Out> for Seq<SM1, SM2>
where
    SM1: StateMachine<In, Out>,
    SM2: StateMachine<In, Out>,
{
    type State = Either<SM1::State, SM2::State>;

    fn start_state(&self) -> Self::State {
        Either::Left(self.first_machine.start_state())
    }

    fn done(&self, state: Self::State) -> bool {
        if let Either::Rigth(state) = state {
            self.second_machine.done(state)
        } else {
            false
        }
    }

    fn next_values(&self, mut state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        if let Either::Left(s1) = state {
            if !self.first_machine.done(s1.clone()) {
                let (new_s1, out) = self.first_machine.next_values(s1, input);
                return (Either::Left(new_s1), out);
            } else {
                state = Either::Rigth(self.second_machine.start_state());
            }
        }

        if let Either::Rigth(s2) = state.clone() {
            if !self.second_machine.done(s2.clone()) {
                let (new_s2, out) = self.second_machine.next_values(s2, input);
                return (Either::Rigth(new_s2), out);
            }
        }
        (state, None)
    }
}

#[derive(Debug)]
pub struct If<SM1, SM2, P> {
    first_machine: SM1,
    second_machine: SM2,
    pred: P,
}

impl<In, Out, SM1, SM2, P> StateMachine<In, Out> for If<SM1, SM2, P>
where
    In: Clone,
    P: Fn(In) -> bool,
    SM1: StateMachine<In, Out>,
    SM2: StateMachine<In, Out>,
{
    type State = Option<Either<SM1::State, SM2::State>>;

    fn start_state(&self) -> Self::State {
        None
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let state = match state {
            Some(state) => state,
            None => {
                if let Some(true) = input.clone().map(|input| (self.pred)(input)) {
                    Either::Left(self.first_machine.start_state())
                } else {
                    Either::Rigth(self.second_machine.start_state())
                }
            }
        };

        match state {
            Either::Left(s1) => {
                let (new_s1, out) = self.first_machine.next_values(s1, input.clone());
                (Some(Either::Left(new_s1)), out)
            }
            Either::Rigth(s2) => {
                let (new_s2, out) = self.second_machine.next_values(s2, input.clone());
                (Some(Either::Rigth(new_s2)), out)
            }
        }
    }
}

pub struct Mux<SM1, SM2, P> {
    first_machine: SM1,
    second_machine: SM2,
    pred: P,
}

impl<In, Out, SM1, SM2, P> StateMachine<In, Out> for Mux<SM1, SM2, P>
where
    In: Clone,
    P: Fn(In) -> bool,
    SM1: StateMachine<In, Out>,
    SM2: StateMachine<In, Out>,
{
    type State = (SM1::State, SM2::State);

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (s1, s2) = state;
        let (new_s1, out1) = self.first_machine.next_values(s1, input.clone());
        let (new_s2, out2) = self.second_machine.next_values(s2, input.clone());
        (
            (new_s1, new_s2),
            if let Some(true) = input.map(|input| (self.pred)(input)) {
                out1
            } else {
                out2
            },
        )
    }
}

pub struct Switch<SM1, SM2, P> {
    first_machine: SM1,
    second_machine: SM2,
    pred: P,
}

impl<In, Out, SM1, SM2, P> StateMachine<In, Out> for Switch<SM1, SM2, P>
where
    In: Clone,
    P: Fn(In) -> bool,
    SM1: StateMachine<In, Out>,
    SM2: StateMachine<In, Out>,
{
    type State = (SM1::State, SM2::State);

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (s1, s2) = state;
        if let Some(true) = input.clone().map(|input| (self.pred)(input)) {
            let (new_s1, out) = self.first_machine.next_values(s1, input);
            ((new_s1, s2), out)
        } else {
            let (new_s2, out) = self.second_machine.next_values(s2, input);
            ((s1, new_s2), out)
        }
    }
}
// pub struct FeedbackOp<SM1, SM2, Op, I> {
//     first_machine: SM1,
//     second_machine: SM2,
//     op: Op,
//     _phantom: PhantomData<I>,
// }

// impl<I, I1, SM1, SM2, Op> StateMachine<I> for FeedbackOp<SM1, SM2, Op, I1>
// where
//     I: Clone,
//     I1: Clone,
//     SM1::Output: Clone,
//     Op: Fn(I, SM2::Output) -> I1,
//     SM1: StateMachine<I1>,
//     SM2: StateMachine<SM1::Output>,
//     SM1::State: Clone,
//     SM2::State: Clone,
// {
//     type State = (SM1::State, SM2::State);
//     type Output = SM1::Output;

//     fn start_state(&self) -> Self::State {
//         (
//             self.first_machine.start_state(),
//             self.second_machine.start_state(),
//         )
//     }

//     fn next_values(
//         &self,
//         state: Self::State,
//         input: Option<I>,
//     ) -> (Self::State, Option<Self::Output>) {
//         let (_, output1) = self.first_machine.next_values(state.0.clone(), None);
//         let (new_state2, output2) = self
//             .second_machine
//             .next_values(state.1.clone(), output1.clone());
//         let (new_state1, _) = self
//             .first_machine
//             .next_values(state.0, input.zip(output2).map(|(i, i2)| (self.op)(i, i2)));
//         ((new_state1, new_state2), output1)
//     }
// }

// pub struct Parallel2<SM1, SM2> {
//     machine1: SM1,
//     machine2: SM2,
// }

// impl<I1, I2, SM1, SM2> StateMachine<(I1, I2)> for Parallel2<SM1, SM2>
// where
//     I1: Clone,
//     I2: Clone,
//     SM1: StateMachine<I1>,
//     SM2: StateMachine<I2>,
// {
//     type State = (SM1::State, SM2::State);
//     type Output = (SM1::Output, SM2::Output);

//     fn start_state(&self) -> Self::State {
//         (self.machine1.start_state(), self.machine2.start_state())
//     }

//     fn next_values(
//         &self,
//         state: Self::State,
//         input: Option<(I1, I2)>,
//     ) -> (Self::State, Option<Self::Output>) {
//         let (i1, i2) = input.unzip();
//         let (new_state1, output1) = self.machine1.next_values(state.0, i1);
//         let (new_state2, output2) = self.machine2.next_values(state.1, i2);
//         let new_state = (new_state1, new_state2);
//         match (output1, output2) {
//             (None, None) | (None, Some(_)) | (Some(_), None) => (new_state, None),
//             (Some(o1), Some(o2)) => (new_state, Some((o1, o2))),
//         }
//     }
// }
pub struct Feedback2<SM> {
    machine: SM,
}

impl<In, Out, SM> StateMachine<In, Out> for Feedback2<SM>
where
    SM: StateMachine<(In, Out), Out>,
    Out: Clone,
{
    type State = SM::State;

    fn start_state(&self) -> Self::State {
        self.machine.start_state()
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (_, output) = self.machine.next_values(state.clone(), None);
        let (new_state, _) = self.machine.next_values(state, input.zip(output.clone()));
        (new_state, output)
    }
}

pub struct Feedback<SM> {
    machine: SM,
}

impl<I, SM> StateMachine<I, I> for Feedback<SM>
where
    I: Clone,
    SM: StateMachine<I, I>,
{
    type State = SM::State;

    fn start_state(&self) -> Self::State {
        self.machine.start_state()
    }

    fn next_values(&self, state: Self::State, _input: Option<I>) -> (Self::State, Option<I>) {
        let (_, output) = self.machine.next_values(state.clone(), None);
        let (new_state, _) = self.machine.next_values(state, output.clone());
        (new_state, output)
    }
}

pub struct Parallel<SM1, SM2> {
    machine1: SM1,
    machine2: SM2,
}

impl<In, Out1, Out2, SM1, SM2> StateMachine<In, (Out1, Out2)> for Parallel<SM1, SM2>
where
    In: Clone,
    SM1: StateMachine<In, Out1>,
    SM2: StateMachine<In, Out2>,
{
    type State = (SM1::State, SM2::State);

    fn start_state(&self) -> Self::State {
        (self.machine1.start_state(), self.machine2.start_state())
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<In>,
    ) -> (Self::State, Option<(Out1, Out2)>) {
        let (new_state1, output1) = self.machine1.next_values(state.0, input.clone());
        let (new_state2, output2) = self.machine2.next_values(state.1, input);
        let new_state = (new_state1, new_state2);
        match (output1, output2) {
            (None, None) | (None, Some(_)) | (Some(_), None) => (new_state, None),
            (Some(o1), Some(o2)) => (new_state, Some((o1, o2))),
        }
    }
}

pub struct Cascade<SM1, SM2, O> {
    first_machine: SM1,
    second_machine: SM2,
    _phantom: PhantomData<O>,
}

impl<In, O, Out, SM1, SM2> StateMachine<In, Out> for Cascade<SM1, SM2, O>
where
    SM1: StateMachine<In, O>,
    SM2: StateMachine<O, Out>,
{
    type State = (SM1::State, SM2::State);

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn done(&self, state: Self::State) -> bool {
        let (s1, s2) = state;
        self.first_machine.done(s1) || self.second_machine.done(s2)
    }

    fn next_values(&self, state: Self::State, input: Option<In>) -> (Self::State, Option<Out>) {
        let (new_state1, output1) = self.first_machine.next_values(state.0, input);
        let (new_state2, output2) = self.second_machine.next_values(state.1, output1);
        ((new_state1, new_state2), output2)
    }
}
