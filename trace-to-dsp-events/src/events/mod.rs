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




pub trait EventData : Default + Debug + Clone + Display {
    fn has_influence_at(&self, index: Real) -> bool {
        true
    }
    fn get_intensity_at(&self, index: Real) -> Real {
        Real::default()
    }
}
pub trait Event : Debug + Clone + Display {
    fn get_time(&self) -> Real;
    fn has_influence_at(&self, index: Real) -> bool;
    fn get_intensity_at(&self, index: Real) -> Real;
}
pub trait EventWithData : Event {
    type DataType : EventData;

    fn get_data(&self) -> &Self::DataType;
    fn take_data(self) -> Self::DataType;
}
    



#[derive(Default,Debug,Clone)]
pub struct SimpleEvent<D> where D : EventData {
    pub time: Real,
    pub data : D,
}
impl<D> Event for SimpleEvent<D> where D : EventData {
    fn get_time(&self) -> Real {
        self.time
    }
    fn has_influence_at(&self, index: Real) -> bool {
        self.data.has_influence_at(index)
    }
    fn get_intensity_at(&self, index: Real) -> Real {
        self.data.get_intensity_at(index)
    }
}

impl<D> EventWithData for SimpleEvent<D> where D : EventData {
    type DataType = D;
    
    fn get_data(&self) -> &D {
        &self.data
    }
    
    fn take_data(self) -> D {
        self.data
    }
}

impl<D> SimpleEvent<D> where D : EventData {
    pub fn new(time : Real, data : D) -> Self {
        SimpleEvent {data, time}
    }
}

impl<C> Display for SimpleEvent<C> where C : EventData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0},{1};", self.time, self.data))
    }
}

/*

#[derive(Default,Debug,Clone)]
pub struct BoundedEvent<C> where C : EventData {
    pub class : C,
    pub time : Real,
    pub time_value: TimeValue,
    pub bounds: (TimeValue,TimeValue),
}
impl<C> Event for BoundedEvent<C> where C : EventClass {
    fn get_time(&self) -> Real {
        self.time
    }
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
        BoundedEvent {class, time: time_value.time, time_value, bounds,}
    }
}

impl<C> Display for BoundedEvent<C> where C : EventClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (start,end) = self.bounds;
        f.write_fmt(format_args!("{0},{1},{2},{3};", self.class, self.time_value, start, end))
    }
}
*/



#[derive(Debug,Clone)]
pub struct MultipleEvents<E> where E : Event {
    time : Real,
    events : Vec<E>,
}
impl<E> MultipleEvents<E> where E : Event {
    pub fn new(events: Vec<E>, time : Real) -> Self {
        Self { events, time }
    }
    pub fn are_times_consistant(&self) -> bool {
        self.events.iter().all(|e|e.get_time() == self.time)
    }
}
impl<E> Event for MultipleEvents<E> where E : Event {
    fn get_time(&self) -> Real {
        self.time
    }
    fn has_influence_at(&self, index: Real) -> bool {
        self.events.iter().any(|e|e.has_influence_at(index))
    }
    fn get_intensity_at(&self, index: Real) -> Real {
        self.events.iter().map(|e|e.get_intensity_at(index)).sum()
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