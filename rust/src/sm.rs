use std::ops::Add;

pub trait StateMachine<Input: Clone> {
    type State: Clone;
    type Output;

    fn start_state(&self) -> Self::State;
    fn next_values(&self, state: Self::State, input: Input) -> (Self::State, Self::Output);

    fn step(&mut self, state: &mut Self::State, input: Input) -> Self::Output {
        let (new_state, output) = self.next_values(state.clone(), input);
        *state = new_state;
        output
    }

    fn transduce<'a, I: IntoIterator<Item = Input>>(&'a mut self, inputs: I) -> Vec<Self::Output> {
        inputs
            .into_iter()
            .scan(self.start_state(), |current_state, input| {
                Some(self.step(current_state, input))
            })
            .collect()
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

    fn parallel_add<SM2>(self, next_machine: SM2) -> ParallelAdd<Self, SM2>
    where
        Self: Sized,
        SM2: StateMachine<Input>,
    {
        ParallelAdd {
            machine1: self,
            machine2: next_machine,
        }
    }
}

pub struct ParallelAdd<SM1, SM2> {
    machine1: SM1,
    machine2: SM2,
}

impl<I, SM1, SM2, O> StateMachine<I> for ParallelAdd<SM1, SM2>
where
    I: Clone,
    SM1: StateMachine<I>,
    SM2: StateMachine<I>,
    SM1::Output: Add<SM2::Output, Output = O>,
{
    type State = (SM1::State, SM2::State);
    type Output = O;

    fn start_state(&self) -> Self::State {
        (self.machine1.start_state(), self.machine2.start_state())
    }

    fn next_values(&self, state: Self::State, input: I) -> (Self::State, Self::Output) {
        let (new_state1, output1) = self.machine1.next_values(state.0, input.clone());
        let (new_state2, output2) = self.machine2.next_values(state.1, input);
        ((new_state1, new_state2), output1 + output2)
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

    fn next_values(&self, state: Self::State, input: I) -> (Self::State, Self::Output) {
        let (new_state1, output1) = self.machine1.next_values(state.0, input.clone());
        let (new_state2, output2) = self.machine2.next_values(state.1, input);
        ((new_state1, new_state2), (output1, output2))
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

    fn next_values(&self, state: Self::State, input: I1) -> (Self::State, Self::Output) {
        let (new_state1, output1) = self.first_machine.next_values(state.0, input);
        let (new_state2, output2) = self.second_machine.next_values(state.1, output1);
        ((new_state1, new_state2), output2)
    }
}
