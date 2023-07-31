use std::fmt::Display;
use std::slice::Iter;
use common::Intensity;
use common::Time;
use std::fmt::Debug;
use crate::Detector;
use crate::EventIter;
use crate::{Integer,Real};
/*
#[derive(Default,Debug,Clone,Copy)]
pub struct FuzzyReal {
    value: Real,
    uncertainty: Real,
}

impl FuzzyReal {
    pub fn from_real(value: Real) -> Self { Self {value, uncertainty: 0.} }
    pub fn new(value: Real, uncertainty : Real) -> Self { Self {value, uncertainty} }
    pub fn get_central(&self) -> Real { self.value }
    pub fn get_uncertainty(&self) -> Real { self.uncertainty }
}
impl Display for FuzzyReal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}~{1}",self.value,self.uncertainty,))
    }
}
*/


#[derive(Default,Debug,Clone,Copy)]
pub struct TimeValue {
    pub time : Real,
    pub value : Real,
}
impl TimeValue {
    pub fn new(time: Real, value : Real) -> Self { Self {time, value} }
    pub fn from_exact( time : Real, value : Real ) -> Self { Self {
        time: Real::from(time),
        value: Real::from(value),
    }}
}
impl Display for TimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}:{1}",self.time,self.value,))
    }
}




pub trait EventClass : Default + Debug + Clone + Display {}
pub trait Event : Debug + Clone + Display {
    fn has_influence_at(&self, index: Real) -> bool;
    fn get_intensity(&self, index: Real) -> Real;
}



#[derive(Default,Debug,Clone)]
pub struct SimpleEvent<C> where C : EventClass {
    pub class : C,
    pub time_value: TimeValue,
}
impl<C> Event for SimpleEvent<C> where C : EventClass {
    fn has_influence_at(&self, index : Real) -> bool {
        true
    }
    fn get_intensity(&self, index: Real) -> Real {
        0.
    }
}

impl<C> SimpleEvent<C> where C : EventClass {
    pub fn new(class : C, time_value : TimeValue) -> Self {
        SimpleEvent {class, time_value, }
    }
}

impl<C> Display for SimpleEvent<C> where C : EventClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1};", self.class, self.time_value))
    }
}



#[derive(Default,Debug,Clone)]
pub struct BoundedEvent<C> where C : EventClass {
    pub class : C,
    pub time_value: TimeValue,
    pub bounds: (TimeValue,TimeValue),
}
impl<C> Event for BoundedEvent<C> where C : EventClass {
    fn has_influence_at(&self, index : Real) -> bool {
        let (start,end) = self.bounds;
        start.time <= index && index <= end.time
    }
    fn get_intensity(&self, index: Real) -> Real {
        let (start,end) = self.bounds;
        self.time_value.value*Real::exp(-0.5*(self.time_value.time - index).powi(2)/(end.time - start.time).powi(2))
    }
}

impl<C> BoundedEvent<C> where C : EventClass {
    pub fn new(class : C, time_value : TimeValue, bounds : (TimeValue,TimeValue)) -> Self {
        BoundedEvent {class, time_value, bounds,}
    }
}

impl<C> Display for BoundedEvent<C> where C : EventClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start,end) = self.bounds;
        f.write_fmt(format_args!("{0},{1},{2},{3};", self.class, self.time_value, start, end))
    }
}




#[derive(Debug,Clone)]
pub struct MultipleEvents<E> where E : Event {
    events : Vec<E>,
}
impl<E> MultipleEvents<E> where E : Event {
    pub fn new(events: Vec<E>) -> Self {
        Self { events }
    }
}
impl<E> Event for MultipleEvents<E> where E : Event {
    fn has_influence_at(&self, index : Real) -> bool {
        true
    }
    fn get_intensity(&self, index: Real) -> Real {
        0.
    }
}
impl<E> Display for MultipleEvents<E> where E : Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for event in &self.events {
            f.write_fmt(format_args!("{event}"))?;
        }
        Ok(())
    }
}


pub struct MultipleEventsIntoIterator<E> where E : Event {
    source : std::vec::IntoIter<E>,
}

impl<E> Iterator for MultipleEventsIntoIterator<E> where E : Event {
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        self.source.next()
    }

}

impl<E> IntoIterator for MultipleEvents<E> where
    E : Event
{
    type Item = E;
    type IntoIter =  std::vec::IntoIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        self.events.into_iter()
    }
}