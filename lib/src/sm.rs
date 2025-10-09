use std::marker::PhantomData;

#[derive(Clone)]
pub enum Either<L, R> {
    Left(L),
    Rigth(R),
}

pub trait StateFullMachine<I, O> {
    fn reset(&mut self);
    fn step(&mut self, input: Option<I>) -> Option<O>;
    fn is_done(&self) -> bool;

    fn transduce<'a, II: IntoIterator<Item = I>>(&'a mut self, inputs: II) -> Vec<O>
    where
        Self: Sized,
    {
        self.reset();
        inputs
            .into_iter()
            .map_while(|input| {
                if self.is_done() {
                    None
                } else {
                    self.step(Some(input))
                }
            })
            .collect()
    }

    fn run(&mut self, n: usize) -> Vec<O> {
        self.reset();
        (0..n)
            .map_while(|_input| {
                if self.is_done() {
                    None
                } else {
                    self.step(None)
                }
            })
            .collect()
    }
}

impl<I, O, F> StateFullMachine<I, O> for F
where
    F: FnMut(I) -> O,
{
    fn reset(&mut self) {}
    fn step(&mut self, input: Option<I>) -> Option<O> {
        input.map(|input| self(input))
    }
    fn is_done(&self) -> bool {
        false
    }
}

pub struct StateFull<I, O, SM>(SM::State, SM)
where
    I: Clone,
    SM: StateMachine<I, Output = O>;

impl<I, O, SM> StateFullMachine<I, O> for StateFull<I, O, SM>
where
    I: Clone,
    SM: StateMachine<I, Output = O>,
    SM::State: Clone,
{
    fn reset(&mut self) {
        self.0 = self.1.start_state();
    }

    fn step(&mut self, input: Option<I>) -> Option<O> {
        let (new_state, output) = self.1.next_values(self.0.clone(), input);
        self.0 = new_state;
        output
    }

    fn is_done(&self) -> bool {
        self.1.done(self.0.clone())
    }
}

impl<I, O, SM> StateFull<I, O, SM>
where
    I: Clone,
    SM: StateMachine<I, Output = O>,
    SM::State: Clone,
{
}

pub trait StateMachine<Input: Clone> {
    type State: Clone;
    type Output;

    fn start_state(&self) -> Self::State;
    fn next_values(
        &self,
        state: Self::State,
        input: Option<Input>,
    ) -> (Self::State, Option<Self::Output>);

    fn done(&self, _state: Self::State) -> bool {
        false
    }

    fn into_state_full(self) -> StateFull<Input, Self::Output, Self>
    where
        Self: Sized,
    {
        StateFull(self.start_state(), self)
    }

    fn cascade<I2, SM2>(self, next_machine: SM2) -> Cascade<Self, SM2>
    where
        Self: Sized,
        I2: Clone,
        SM2: StateMachine<I2>,
        Self: StateMachine<Input, Output = I2>,
    {
        Cascade {
            first_machine: self,
            second_machine: next_machine,
        }
    }

    fn parallel<SM2>(self, next_machine: SM2) -> Parallel<Self, SM2>
    where
        Self: Sized,
        SM2: StateMachine<Input>,
    {
        Parallel {
            machine1: self,
            machine2: next_machine,
        }
    }

    fn parallel2<I2, SM2>(self, next_machine: SM2) -> Parallel2<Self, SM2>
    where
        Self: Sized,
        I2: Clone,
        SM2: StateMachine<I2>,
    {
        Parallel2 {
            machine1: self,
            machine2: next_machine,
        }
    }

    fn feedback(self) -> Feedback<Self>
    where
        Self: Sized,
    {
        Feedback { machine: self }
    }

    fn feedback2(self) -> Feedback2<Self, Input>
    where
        Self: Sized,
    {
        Feedback2 {
            machine: self,
            _phantom: PhantomData,
        }
    }

    fn feedback_op<SM, Op>(self, machine: SM, op: Op) -> FeedbackOp<Self, SM, Op, Input>
    where
        Self: Sized,
    {
        FeedbackOp {
            first_machine: self,
            second_machine: machine,
            op,
            _phantom: PhantomData,
        }
    }

    fn switch<SM, P>(self, machine: SM, pred: P) -> Switch<Self, SM, P>
    where
        Self: Sized,
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
    {
        If {
            first_machine: self,
            second_machine: machine,
            pred,
        }
    }
}

pub struct If<SM1, SM2, P> {
    first_machine: SM1,
    second_machine: SM2,
    pred: P,
}

impl<I, SM1, SM2, P> StateMachine<I> for If<SM1, SM2, P>
where
    I: Clone,
    P: Fn(I) -> bool,
    SM1: StateMachine<I>,
    SM2: StateMachine<I, Output = SM1::Output>,
{
    type State = Option<Either<SM1::State, SM2::State>>;
    type Output = SM1::Output;

    fn start_state(&self) -> Self::State {
        None
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
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

impl<I, SM1, SM2, P> StateMachine<I> for Mux<SM1, SM2, P>
where
    I: Clone,
    P: Fn(I) -> bool,
    SM1: StateMachine<I>,
    SM2: StateMachine<I, Output = SM1::Output>,
{
    type State = (SM1::State, SM2::State);
    type Output = SM1::Output;

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
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

impl<I, SM1, SM2, P> StateMachine<I> for Switch<SM1, SM2, P>
where
    I: Clone,
    P: Fn(I) -> bool,
    SM1: StateMachine<I>,
    SM2: StateMachine<I, Output = SM1::Output>,
{
    type State = (SM1::State, SM2::State);
    type Output = SM1::Output;

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
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
pub struct FeedbackOp<SM1, SM2, Op, I> {
    first_machine: SM1,
    second_machine: SM2,
    op: Op,
    _phantom: PhantomData<I>,
}

impl<I, I1, SM1, SM2, Op> StateMachine<I> for FeedbackOp<SM1, SM2, Op, I1>
where
    I: Clone,
    I1: Clone,
    SM1::Output: Clone,
    Op: Fn(I, SM2::Output) -> I1,
    SM1: StateMachine<I1>,
    SM2: StateMachine<SM1::Output>,
{
    type State = (SM1::State, SM2::State);
    type Output = SM1::Output;

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        let (_, output1) = self.first_machine.next_values(state.0.clone(), None);
        let (new_state2, output2) = self
            .second_machine
            .next_values(state.1.clone(), output1.clone());
        let (new_state1, _) = self
            .first_machine
            .next_values(state.0, input.zip(output2).map(|(i, i2)| (self.op)(i, i2)));
        ((new_state1, new_state2), output1)
    }
}

pub struct Feedback2<SM, I> {
    machine: SM,
    _phantom: PhantomData<I>,
}

impl<I1, I2, SM> StateMachine<I1> for Feedback2<SM, (I1, I2)>
where
    I1: Clone,
    I2: Clone,
    SM: StateMachine<(I1, I2), Output = I2>,
    SM::State: Clone,
{
    type State = SM::State;
    type Output = SM::Output;

    fn start_state(&self) -> Self::State {
        self.machine.start_state()
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I1>,
    ) -> (Self::State, Option<Self::Output>) {
        let (_, output) = self.machine.next_values(state.clone(), None);
        let (new_state, _) = self.machine.next_values(state, input.zip(output.clone()));
        (new_state, output)
    }
}

pub struct Feedback<SM> {
    machine: SM,
}

impl<I, SM> StateMachine<I> for Feedback<SM>
where
    I: Clone,
    SM: StateMachine<I, Output = I>,
    SM::State: Clone,
{
    type State = SM::State;
    type Output = SM::Output;

    fn start_state(&self) -> Self::State {
        self.machine.start_state()
    }

    fn next_values(
        &self,
        state: Self::State,
        _input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        let (_, output) = self.machine.next_values(state.clone(), None);
        let (new_state, _) = self.machine.next_values(state, output.clone());
        (new_state, output)
    }
}

pub struct Parallel2<SM1, SM2> {
    machine1: SM1,
    machine2: SM2,
}

impl<I1, I2, SM1, SM2> StateMachine<(I1, I2)> for Parallel2<SM1, SM2>
where
    I1: Clone,
    I2: Clone,
    SM1: StateMachine<I1>,
    SM2: StateMachine<I2>,
{
    type State = (SM1::State, SM2::State);
    type Output = (SM1::Output, SM2::Output);

    fn start_state(&self) -> Self::State {
        (self.machine1.start_state(), self.machine2.start_state())
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<(I1, I2)>,
    ) -> (Self::State, Option<Self::Output>) {
        let (i1, i2) = input.unzip();
        let (new_state1, output1) = self.machine1.next_values(state.0, i1);
        let (new_state2, output2) = self.machine2.next_values(state.1, i2);
        let new_state = (new_state1, new_state2);
        match (output1, output2) {
            (None, None) | (None, Some(_)) | (Some(_), None) => (new_state, None),
            (Some(o1), Some(o2)) => (new_state, Some((o1, o2))),
        }
    }
}

pub struct Parallel<SM1, SM2> {
    machine1: SM1,
    machine2: SM2,
}

impl<I, SM1, SM2> StateMachine<I> for Parallel<SM1, SM2>
where
    I: Clone,
    SM1: StateMachine<I>,
    SM2: StateMachine<I>,
{
    type State = (SM1::State, SM2::State);
    type Output = (SM1::Output, SM2::Output);

    fn start_state(&self) -> Self::State {
        (self.machine1.start_state(), self.machine2.start_state())
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I>,
    ) -> (Self::State, Option<Self::Output>) {
        let (new_state1, output1) = self.machine1.next_values(state.0, input.clone());
        let (new_state2, output2) = self.machine2.next_values(state.1, input);
        let new_state = (new_state1, new_state2);
        match (output1, output2) {
            (None, None) | (None, Some(_)) | (Some(_), None) => (new_state, None),
            (Some(o1), Some(o2)) => (new_state, Some((o1, o2))),
        }
    }
}

pub struct Cascade<SM1, SM2> {
    first_machine: SM1,
    second_machine: SM2,
}

impl<I1, I2, SM1, SM2> StateMachine<I1> for Cascade<SM1, SM2>
where
    I1: Clone,
    I2: Clone,
    SM1: StateMachine<I1, Output = I2>,
    SM2: StateMachine<I2>,
{
    type State = (SM1::State, SM2::State);
    type Output = SM2::Output;

    fn start_state(&self) -> Self::State {
        (
            self.first_machine.start_state(),
            self.second_machine.start_state(),
        )
    }

    fn next_values(
        &self,
        state: Self::State,
        input: Option<I1>,
    ) -> (Self::State, Option<Self::Output>) {
        let (new_state1, output1) = self.first_machine.next_values(state.0, input);
        let (new_state2, output2) = self.second_machine.next_values(state.1, output1);
        ((new_state1, new_state2), output2)
    }
}
