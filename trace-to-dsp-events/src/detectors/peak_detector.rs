use std::fmt::Display;

use super::event::{TimeValue,EventClass,SimpleEvent};
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
    type EventType = SimpleEvent<Class>;

    fn signal(&mut self, time : Real, value: Real) -> Option<Self::EventType> {
        let mut return_value = Option::<SimpleEvent<Class>>::default();
        if self.prev_prev_value < self.prev_value && value < self.prev_value {
            let peak    = TimeValue{ time: Real::from(time - 1.), value: Real::from(self.prev_value)};
            return_value = Some(SimpleEvent::new(
                    Class::LocalMax,
                    peak,
                ));
        } else if self.prev_prev_value > self.prev_value && value > self.prev_value {
            let peak    = TimeValue{ time: Real::from(time - 1.), value: Real::from(self.prev_value)};
            return_value = Some(SimpleEvent::new(
                    Class::LocalMin,
                    peak,
                ));
        }
        self.prev_prev_value = self.prev_value;
        self.prev_value = value;
        return_value
    }
}
