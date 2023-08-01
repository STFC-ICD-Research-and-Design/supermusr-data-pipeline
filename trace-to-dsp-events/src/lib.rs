// Code from https://github.com/swizard0/smoothed_z_score/blob/master/README.md


/*
iterators of raw trace data have the trait EventFilter<I,S,D> implemented
The events method consumes a raw trace iterator and emits an EventIter iterator
A detector is a struct that 

I is an iterator to the enumerated raw trace data, S is the detector signal type and D is the detector.

*/

pub mod detectors;

use std::{collections::VecDeque, iter::Peekable, marker::PhantomData, slice::Iter};

use common::Intensity;

pub mod events;
use events::{Event, MultipleEvents};
pub use detectors::{Detector, peak_detector,event_detector};

pub mod trace_iterators;
pub use trace_iterators::RealArray;

pub mod window;
pub use window::smoothing_window::SmoothingWindow;



pub type Real = f64;
pub type Integer = i16;

pub mod processing {
    use super::*;
    pub fn make_enumerate_real((i,v) : (usize, &Intensity)) -> (Real,Real) { (i as Real, *v as Real) }
    pub fn make_enumerate_integeral((i,v) : (Real,Real)) -> (usize, Integer) { (i as usize, v as Integer) }
}







pub struct EventIter<I,D> where I: Iterator<Item = (D::TimeType,D::ValueType)>, D : Detector {
    source : I,
    detector : D,
}

impl<I,D> Iterator for EventIter<I,D> where
    I: Iterator<Item = (D::TimeType,D::ValueType)>,
    D : Detector
{
    type Item = D::EventType;
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(item) = self.source.next() {
            if let Some(event) = self.detector.signal(item.0,item.1) {
                return Some(event);
            }
        }
        None
    }
}

/*
impl<'a, I,D,E> EventIter<I,D> where
        I: Iterator<Item = (D::TimeType, D::ValueType)>,
        D : Detector<EventType = MultipleEvents<E>>,
        E : Event
{
    pub fn unpack_multiple_events(self) -> MultiEventIter<'a, I,D,E> {
        //let events = self.source.next();
        MultiEventIter { source: self.source, events: None, phantom: PhantomData}
    }
}

pub struct MultiEventIter<'a, I,D,E> where
    I: Iterator<Item = (D::TimeType, D::ValueType)>,
    D : Detector<EventType = MultipleEvents<E>>,
    E : Event
{
    source : I,
    events : Option<Iter<'a, E>>,
    phantom : PhantomData<D>,
}

impl<'a, I,D,E> Iterator for MultiEventIter<'a, I,D,E> where
    I: Iterator<Item = (D::TimeType, D::ValueType)>,
    D : Detector<EventType = MultipleEvents<E>>,
    E : Event
{
    type Item = E;
    fn next(&mut self) -> Option<Self::Item> {
        self.events = match self.events {
            Some(event) => event.next(),
            None => self.source.next().map(|e|e.1.iter()),
        };
        None
    }
}

*/



pub trait EventFilter<I,D> where I: Iterator<Item = (D::TimeType,D::ValueType)>, D : Detector {
    fn events(self, detector : D) -> EventIter<I,D>;
}

impl<I,D> EventFilter<I,D> for I where I : Iterator<Item = (D::TimeType,D::ValueType)>, D : Detector {
    fn events(self, detector: D) -> EventIter<I,D> {
        EventIter { source: self, detector }
    }
}



pub struct TraceMakerIter<I,E> where I: Iterator<Item = E>, E : Event {
    source : Peekable<I>,
    end : usize,

    next_event : Option<E>,
    index : usize,
    events : VecDeque<E>
}

impl<I,E> TraceMakerIter<I,E> where I: Iterator<Item = E>, E : Event {
    fn new(source: I, end : usize) -> Self {
        let mut itr = Self { source: source.peekable(), end,
            next_event: Option::<E>::default(),
            index : usize::default(),
            events : VecDeque::<E>::default(),
        };
        itr.next_event = itr.source.next();
        itr
    }
}

impl<I,E> Iterator for TraceMakerIter<I,E> where I: Iterator<Item = E>, E : Event {
    type Item = (Real,Real);
    fn next(&mut self) -> Option<Self::Item> {
        if self.index == self.end {
            return None;
        }
        //  Remove old events that are no longer influential
        while let Some(e) = self.events.front() {
            if e.has_influence_at(self.index as Real) {
                break;
            } else {
                self.events.pop_front();
            }
        }
        //  Append new events that are influencial
        while let Some(e) = self.source.peek() {
            if e.has_influence_at(self.index as Real) {
                if let Some(e) = self.source.next() {
                    self.events.push_back(e);
                } else {
                    panic!("A peek led me wrong");
                    //break; // This should never happen
                }
            } else {
                break;
            }
        }

        
        self.index += 1;
        Some(((self.index - 1) as Real, self.events.iter().map(|e|e.get_intensity_at((self.index - 1) as Real)).sum()))
    }
}




pub trait TraceMakerFilter<I,E> where I: Iterator<Item = E>, E : Event {
    fn trace(self, end : usize) -> TraceMakerIter<I,E>;
}

impl<I,E> TraceMakerFilter<I,E> for I where I: Iterator<Item = E>, E : Event {
    fn trace(self, end : usize) -> TraceMakerIter<I,E> {
        TraceMakerIter::new(self,end)
    }
}




#[cfg(test)]
mod tests {
    use common::Intensity;
    use crate::window::WindowFilter;
    use crate::window::composite::CompositeWindow;

    use super::trace_iterators::finite_difference::FiniteDifferencesFilter;

    use super::{event_detector::EventsDetector, EventFilter,Real};

    #[test]
    fn sample_data() {
        let input = vec![
            1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.0, 1.1, 1.0,
            1.0, 1.0, 1.0, 1.1, 0.9, 1.0, 1.1, 1.0, 1.0, 0.9, 1.0, 1.1, 1.0, 1.0, 1.1, 1.0, 0.8, 0.9, 1.0,
            1.2, 0.9, 1.0, 1.0, 1.1, 1.2, 1.0, 1.5, 1.0, 3.0, 2.0, 5.0, 3.0, 2.0, 1.0, 1.0, 1.0, 0.9, 1.0,
            1.0, 3.0, 2.6, 4.0, 3.0, 3.2, 2.0, 1.0, 1.0, 0.8, 4.0, 4.0, 2.0, 2.5, 1.0, 1.0, 1.0
        ];
        let output: Vec<_> = input.iter().map(|x|(x*1000.) as Intensity)
            .into_iter()
            .enumerate()
            .map(|(i,v)|(i as Real,v as Real))
            .finite_differences()
            .window(CompositeWindow::trivial())
            .events(EventsDetector::new([10.0, 2.0]))
            .collect();
        for line in output {
            println!("{line:?}")
        }
    }
}