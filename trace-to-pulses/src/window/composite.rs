use std::fmt::{Debug, Display};
use std::marker::PhantomData;

use crate::tracedata::Temporal;
use crate::TracePair;

use crate::window::Window;

#[derive(Default, Clone, Debug)]
pub struct DoublingWindow<T, V>
where
    T: Temporal,
    V: Default + Copy + Display + Debug,
{
    pair: TracePair<V, V>,
    phanton: PhantomData<T>,
}
impl<T, V> DoublingWindow<T, V>
where
    T: Temporal,
    V: Default + Copy + Display + Debug,
{
    pub fn new() -> Self {
        DoublingWindow::default()
    }
}
impl<T, V> Window for DoublingWindow<T, V>
where
    T: Temporal,
    V: Default + Copy + Display + Debug,
{
    type TimeType = T;
    type InputType = V;
    type OutputType = TracePair<V, V>;

    fn push(&mut self, value: Self::InputType) -> bool {
        self.pair = TracePair(value, value);
        true
    }
    fn stats(&self) -> Option<Self::OutputType> {
        Some(self.pair)
    }
    fn apply_time_shift(&self, time: T) -> T {
        time
    }
}

#[derive(Default, Clone, Debug)]
pub struct CompositeWindow<W1, W2, T>
where
    T: Temporal,
    W1: Window<TimeType = T>,
    W2: Window<TimeType = T>,
{
    window1: W1,
    window2: W2,
}
impl<W1, W2, T> CompositeWindow<W1, W2, T>
where
    T: Temporal,
    W1: Window<TimeType = T>,
    W2: Window<TimeType = T>,
{
    pub fn new(window1: W1, window2: W2) -> Self {
        CompositeWindow { window1, window2 }
    }
}
impl<W1, W2, T> Window for CompositeWindow<W1, W2, T>
where
    T: Temporal,
    W1: Window<TimeType = T>,
    W2: Window<TimeType = T>,
    W1::InputType: Default + Clone + Debug + Display,
    W2::InputType: Default + Clone + Debug + Display,
    W1::OutputType: Default + Clone + Debug + Display,
    W2::OutputType: Default + Clone + Debug + Display,
{
    type TimeType = T;
    type InputType = TracePair<W1::InputType, W2::InputType>;
    type OutputType = TracePair<W1::OutputType, W2::OutputType>;

    fn push(&mut self, value: Self::InputType) -> bool {
        self.window1.push(value.0) && self.window2.push(value.1)
    }
    fn stats(&self) -> Option<Self::OutputType> {
        Some(TracePair(self.window1.stats()?, self.window2.stats()?))
    }
    fn apply_time_shift(&self, time: T) -> T {
        time
    }
}
