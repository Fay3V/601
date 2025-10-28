use std::{
    iter::Sum,
    ops::{Add, Mul},
};

pub trait Signal {
    type Out;
    fn sample(&mut self, n: i32) -> Self::Out;

    fn scale<G>(mut self, gain: G) -> impl Signal<Out = Self::Out>
    where
        Self: Sized,
        G: Mul<Self::Out, Output = Self::Out> + Copy,
    {
        move |n| gain * self.sample(n)
    }

    fn delay<const R: usize>(mut self) -> impl Signal<Out = Self::Out>
    where
        Self: Sized,
    {
        move |n| self.sample(n - (R as i32))
    }

    fn add<S>(mut self, mut sig: S) -> impl Signal<Out = Self::Out>
    where
        Self: Sized,
        S: Signal<Out = Self::Out>,
        Self::Out: Add<Self::Out, Output = Self::Out>,
    {
        move |n| self.sample(n) + sig.sample(n)
    }

    fn poly<const DIM: usize, C>(mut self, coefs: [C; DIM]) -> impl Signal<Out = Self::Out>
    where
        Self: Sized,
        C: Clone + Into<Self::Out>,
        Self::Out: Mul<Self::Out, Output = Self::Out>,
        Self::Out: Sum,
    {
        move |n| {
            coefs
                .iter()
                .enumerate()
                .map(|(deg, coef)| self.sample(n - deg as i32) * coef.clone().into())
                .sum()
        }
    }
}

impl<O, F> Signal for F
where
    F: FnMut(i32) -> O,
{
    type Out = O;

    fn sample(&mut self, n: i32) -> Self::Out {
        self(n)
    }
}

pub fn unit() -> impl Signal<Out = f64> {
    |n| if n == 0 { 1.0 } else { 0.0 }
}

pub fn cosine(omega: f64, theta: f64) -> impl Signal<Out = f64> {
    move |n| (omega * n as f64 + theta).cos()
}

pub struct IterSignal<Sg>(Sg, i32);

impl<Sg> IterSignal<Sg>
where
    Sg: Signal,
{
    pub fn new(sig: Sg) -> Self {
        Self(sig, 0)
    }
}

impl<Sg> Iterator for IterSignal<Sg>
where
    Sg: Signal,
{
    type Item = Sg::Out;

    fn next(&mut self) -> Option<Self::Item> {
        let out = self.0.sample(self.1);
        self.1 += 1;
        Some(out)
    }
}
