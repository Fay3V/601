use std::marker::PhantomData;

pub trait StateMachine<Input: Clone> {
    type State: Clone;
    type Output;

    fn start_state(&self) -> Self::State;
    fn next_values(
        &self,
        state: Self::State,
        input: Option<Input>,
    ) -> (Self::State, Option<Self::Output>);

    fn step(&mut self, state: &mut Self::State, input: Option<Input>) -> Option<Self::Output> {
        let (new_state, output) = self.next_values(state.clone(), input);
        *state = new_state;
        output
    }

    fn start<'a>(&'a mut self) -> impl FnMut(Input) -> Option<Self::Output>
    where
        Self: Sized,
        <Self as StateMachine<Input>>::State: 'a,
    {
        let mut current_state = self.start_state();
        move |input| self.step(&mut current_state, Some(input))
    }

    fn transduce<'a, I: IntoIterator<Item = Input>>(&'a mut self, inputs: I) -> Vec<Self::Output>
    where
        Self: Sized,
    {
        inputs.into_iter().map_while(self.start()).collect()
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

    fn feedbackOp<SM, Op>(self, machine: SM, op: Op) -> FeedbackOp<Self, SM, Op, Input>
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
