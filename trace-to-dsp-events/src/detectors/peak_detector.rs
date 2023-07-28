use std::fmt::Display;

use common::Intensity;
use common::Time;
use super::event::{TimeValue,Event,EventClass,SingleEvent,FuzzyReal};
use crate::{Real,Detector};

#[derive(Default,Debug,Clone)]
pub enum Class { #[default]Flat, LocalMax, LocalMin }
impl EventClass for Class {}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self {
                Self::LocalMax => 1,
                Self::Flat => 0,
                Self::LocalMin => -1
            }
        ))
    }
}








#[derive(Default)]
pub struct PeakDetector {
    prev_prev_value : Real,
    prev_value : Real,
}

impl Detector for PeakDetector {
    type TimeType = Real;
    type ValueType = Real;
    type EventType = SingleEvent<Class>;

    fn signal(&mut self, time : Real, value: Real) -> Option<Self::EventType> {
        let mut return_value = Option::<SingleEvent<Class>>::default();
        if self.prev_prev_value < self.prev_value && value < self.prev_value {
            let start   = TimeValue{ time: FuzzyReal::from_real(time - 1.), value: FuzzyReal::from_real(self.prev_prev_value)};
            let peak    = TimeValue{ time: FuzzyReal::from_real(time), value: FuzzyReal::from_real(self.prev_value)};
            let end     = TimeValue{ time: FuzzyReal::from_real(time + 1.), value: FuzzyReal::from_real(value)};
            return_value = Some(SingleEvent::new(
                    Class::LocalMax,
                    peak,
                    Some((start, end))
                ));
        } else if self.prev_prev_value > self.prev_value && value > self.prev_value {
            let start   = TimeValue{ time: FuzzyReal::from_real(time - 1.), value: FuzzyReal::from_real(self.prev_prev_value)};
            let peak    = TimeValue{ time: FuzzyReal::from_real(time), value: FuzzyReal::from_real(self.prev_value)};
            let end     = TimeValue{ time: FuzzyReal::from_real(time + 1.), value: FuzzyReal::from_real(value)};
            return_value = Some(SingleEvent::new(
                    Class::LocalMin,
                    peak,
                    Some((start, end))
                ));
        }
        self.prev_prev_value = self.prev_value;
        self.prev_value = value;
        return_value
    }
}
