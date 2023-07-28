
use std::fmt::Display;

use super::event::{Event,EventClass,TimeValue, SingleEvent, MultipleEvents, FuzzyReal};
use crate::window::Window;
use crate::window::smoothing_window::Stats;
use crate::{Detector, Real, SmoothingWindow, RealArray};

#[derive(Default,Debug,Clone,PartialEq)]
pub enum Class { #[default]Flat, Rising, Falling }
impl EventClass for Class {}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self {
                Self::Rising => 1i32,
                Self::Flat => 0i32,
                Self::Falling => -1i32,
            }
        ))
    }
}

#[derive(Default)]
pub struct ChangeDetector {
    mode : Class,
    start : TimeValue,
    peak : TimeValue,
    trough : TimeValue,

    threshold: Real,
    influence: Real,
    window: SmoothingWindow,
}
impl ChangeDetector {
    pub fn new(lag: usize, threshold: Real, influence: Real) -> ChangeDetector {
        ChangeDetector {
            threshold,
            influence,
            window: SmoothingWindow::new(lag),
            ..Default::default()
        }
    }
}
impl Detector for ChangeDetector {
    type TimeType = Real;
    type ValueType = Real;
    type EventType = SingleEvent<Class>;

    fn signal(&mut self, time : Real, value: Real) -> Option<SingleEvent<Class>> {
        if let Some(Stats{value: _, mean, variance}) = self.window.stats()
        {
            if self.peak.value.get_central() < value {
                self.peak = TimeValue::from_exact(time, value);
            }
            if self.trough.value.get_central() > value {
                self.trough = TimeValue::from_exact(time, value);
            }
            let difference = value - mean;
            let now = TimeValue::new(FuzzyReal::from_real(time as Real), FuzzyReal::new(value, self.threshold * variance.sqrt()));
            let new_mode = if difference.abs() > self.threshold * variance.sqrt() {
                //self.window.push_with_influence(value, self.influence);
                if difference > 0. {
                    Class::Rising
                } else {
                    Class::Falling
                }
            }
            else {
                self.window.push(value);
                Class::Flat
            };
            let event_class = if new_mode == self.mode {
                None
            } else {
                Some(new_mode.clone())
            };
            
            let event = event_class.map(|e|SingleEvent::new(e.clone(), match e { Class::Rising => self.peak, Class::Falling => self.trough, Class::Flat => self.peak,}, Some((self.start.clone(),now.clone()))));
            self.mode = new_mode;
            self.start = now.clone();
            event
        } else {
            self.window.push(value);
            self.peak = TimeValue::from_exact(time, value);
            self.trough = TimeValue::from_exact(time, value);
            None
        }
    }
}

#[derive(Default)]
pub struct FiniteDifferenceChangeDetector<const N : usize> {
    threshold: Real,
    influence: Real,
    detectors: Vec<ChangeDetector>,
}
    
impl<const N : usize> FiniteDifferenceChangeDetector<N> {
    pub fn new(lag: usize, threshold: Real, influence: Real) -> FiniteDifferenceChangeDetector<N> {
        FiniteDifferenceChangeDetector::<N> {
            threshold,
            influence,
            detectors: (0..N).map(|_|ChangeDetector::new(lag,threshold,influence)).collect(),
            ..Default::default()
        }
    }
}



impl<const N : usize> Detector for FiniteDifferenceChangeDetector<N> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type EventType = MultipleEvents<Class>;

    fn signal(&mut self, time : Real, value: RealArray<N>) -> Option<MultipleEvents<Class>> {
        let events : Vec<(usize,SingleEvent<Class>)> = self.detectors
            .iter_mut()
            .enumerate()
            .map(|(i,detector)|detector.signal(time,value[i]).map(|v| (i,v)))
            .flatten()
            .collect();
        if events.is_empty() {
            None
        } else {
            Some(MultipleEvents::new(events))
        }
    }
}