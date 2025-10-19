#[derive(Debug, Clone)]
pub enum Value<T> {
    Undefined,
    Defined(T),
}

impl<T> Value<T> {
    pub fn into_option(self) -> Option<T> {
        match self {
            Value::Undefined => None,
            Value::Defined(v) => Some(v),
        }
    }

    pub fn zip<U>(self, other: Value<U>) -> Value<(T, U)> {
        match (self, other) {
            (Value::Defined(a), Value::Defined(b)) => Value::Defined((a, b)),
            _ => Value::Undefined,
        }
    }
}

pub trait StateMachine<Input, Output> {
    fn next(&mut self, input: Option<Value<Input>>) -> Option<Value<Output>>;

    fn transduce<I>(&mut self, iter: I) -> impl Iterator<Item = Output>
    where
        Self: Sized,
        I: IntoIterator<Item = Input>,
    {
        self.next(None);
        iter.into_iter()
            .map_while(|i| self.next(Some(Value::Defined(i)))?.into_option())
    }

    fn run(&mut self) -> impl Iterator<Item = Output>
    where
        Self: Sized,
    {
        self.next(None);
        (0..).map_while(|_| self.next(Some(Value::Undefined))?.into_option())
    }

    fn cascade<O, SM>(mut self, mut machine: SM) -> impl StateMachine<Input, O>
    where
        Self: Sized,
        SM: StateMachine<Output, O>,
    {
        move |input| match input {
            input @ Some(_) => machine.next(self.next(input)),
            None => {
                self.next(None);
                machine.next(None);
                Some(Value::Undefined)
            }
        }
    }

    fn parallel<O, SM>(mut self, mut machine: SM) -> impl StateMachine<Input, (Output, O)>
    where
        Self: Sized,
        Input: Clone,
        SM: StateMachine<Input, O>,
        Input: std::fmt::Debug,
        Output: std::fmt::Debug,
        O: std::fmt::Debug,
    {
        move |input| match input {
            input @ Some(_) => self
                .next(input.clone())
                .zip(machine.next(input))
                .map(|(o1, o2)| o1.zip(o2)),
            None => {
                self.next(None);
                machine.next(None);
                Some(Value::Undefined)
            }
        }
    }
}

pub trait StateMachineExt<I> {
    fn feedback(self) -> impl StateMachine<I, I>;
}

impl<T, Input> StateMachineExt<Input> for T
where
    T: StateMachine<Input, Input>,
    Input: std::fmt::Debug + Clone,
{
    fn feedback(mut self) -> impl StateMachine<Input, Input>
    where
        Self: Sized,
    {
        move |input| match input {
            input @ Some(_) => {
                dbg!(&input);
                let output = self.next(Some(Value::Undefined));
                dbg!(&output);
                self.next(output.clone());
                output
            }
            None => {
                println!("reseeeeeeeet");
                self.next(None);
                Some(Value::Undefined)
            }
        }
    }
}

impl<I, O, F> StateMachine<I, O> for F
where
    F: FnMut(Option<Value<I>>) -> Option<Value<O>>,
{
    fn next(&mut self, input: Option<Value<I>>) -> Option<Value<O>> {
        self(input)
    }
}

pub fn map_fn_mut<I, O, F>(mut f: F) -> impl StateMachine<I, O>
where
    F: FnMut(Option<I>) -> Option<O>,
{
    move |input: Option<Value<I>>| match input {
        Some(Value::Undefined) => Some(Value::Undefined),
        Some(Value::Defined(input)) => f(Some(input)).map(Value::Defined),
        None => f(None).map(Value::Defined),
    }
}

pub fn map_fn<I, O, F>(f: F) -> impl StateMachine<I, O>
where
    F: Fn(I) -> O,
{
    move |input: Option<Value<I>>| match input {
        Some(Value::Undefined) | None => Some(Value::Undefined),
        Some(Value::Defined(input)) => Some(Value::Defined(f(input))),
    }
}
