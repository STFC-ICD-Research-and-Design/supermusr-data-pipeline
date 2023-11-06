use std::{collections::VecDeque, slice::Iter};

use crate::{
    Real,
    RealArray,
    tracedata::{TraceData, TraceValue},
};
use num::integer::binomial;

use super::iter::{TraceIter, TraceIterType};

#[derive(Default, Clone)]
pub struct RecentPulses<'a, D> where D : EventData
{
    recent_pulses: &'a mut VecDeque<'a,D>,
}

impl<'a,D> TraceIterType for RecentPulses<'a,D> where D : EventData {}

impl<'a,D> RecentPulses<'a,D> where D : EventData {
    pub fn new(recent_pulses: &'a mut VecDeque<D>) -> Self {
        RecentPulses {
            recent_pulses,
        }
    }
}

impl<'a, I> Iterator for TraceIter<RecentPulses<'a, I::Item>,I> where
    I: Iterator,
    I::Item : TraceData,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.child.source.next() {
            Some(value) => Some(value.clone()),
            None => self.source.next(),
        }
    }
}





pub trait RecentPulsesFilter<'a, I> where
    I: Iterator,
    I::Item : TraceData,
{
    fn remove_recent_pulses(self, memory: &'a Vec<I::Item>) -> TraceIter<RecentPulses<'a, I::Item>, I>;
}

impl<'a, I> RecentPulsesFilter<'a, I> for I where
    I: Iterator,
    I::Item : TraceData,
{
    fn remove_recent_pulses(self, memory: &'a Vec<I::Item>) -> TraceIter<RecentPulses<'a, I::Item>, I> {
        TraceIter::new(Memory::new(memory), self)
    }
}



