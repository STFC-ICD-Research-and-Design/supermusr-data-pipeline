use std::fmt::Display;

use crate::events::{TimeValue,EventData,SimpleEvent};
use crate::{Real,Detector};

#[derive(Default,Debug,Clone)]
pub enum Class { #[default]Flat, LocalMax, LocalMin }
impl EventData for Data {}

#[derive(Default,Debug,Clone)]
pub struct Data {
    class : Class,
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self.class {
                Class::LocalMax => 1,
                Class::Flat => 0,
                Class::LocalMin => -1
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
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time : Real, value: Real) -> Option<Self::EventType> {
        let mut return_value = Option::<SimpleEvent<Data>>::default();
        if self.prev_prev_value < self.prev_value && value < self.prev_value {
            let peak    = TimeValue{ time: Real::from(time - 1.), value: Real::from(self.prev_value)};
            return_value = Some(SimpleEvent::new(
                    time - 1.,
                    Data{class:Class::LocalMax}
                ));
        } else if self.prev_prev_value > self.prev_value && value > self.prev_value {
            let peak    = TimeValue{ time: Real::from(time - 1.), value: Real::from(self.prev_value)};
            return_value = Some(SimpleEvent::new(
                    time - 1.,
                    Data{class:Class::LocalMin}
                ));
        }
        self.prev_prev_value = self.prev_value;
        self.prev_value = value;
        return_value
    }
}
