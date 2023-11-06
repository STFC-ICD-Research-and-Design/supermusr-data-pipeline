use std::marker::PhantomData;

use crate::{
    tracedata::{Temporal, TraceValue},
    Real,
};

use super::Window;

pub trait Realisable: From<Real> + Default + Clone {}
impl<T> Realisable for T where T: From<Real> + Default + Clone {}

#[derive(Default, Clone, Copy)]
pub struct TrivialWindow<T, V>
where
    T: Temporal,
    V: TraceValue,
{
    value: V,
    phantom: PhantomData<T>,
}
impl<T, V> Window for TrivialWindow<T, V>
where
    T: Temporal,
    V: TraceValue + Copy,
{
    type TimeType = T;
    type InputType = V;
    type OutputType = V;

    fn push(&mut self, value: V) -> bool {
        self.value = value;
        true
    }
    fn stats(&self) -> Option<Self::OutputType> {
        Some(self.value)
    }
    fn apply_time_shift(&self, time: T) -> T {
        time
    }
}
