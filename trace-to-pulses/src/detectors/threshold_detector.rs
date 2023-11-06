use std::fmt::Display;

use crate::events::Event;
use crate::pulse::{Pulse, TimeValueOptional};
use crate::tracedata::EventData;
use crate::{Detector, Real};

use super::Assembler;

#[derive(Default, Debug, Clone)]
pub struct Data {
    //pub(super) time : Real,
}

impl Display for Data {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
        //f.write_fmt(format_args!("{0},{1},{2}", self.time.unwrap_or_default(), self.start.unwrap_or_default(), self.end.unwrap_or_default()))
    }
}
impl EventData for Data {}

#[derive(Default, Debug, Clone)]
pub struct ThresholdDuration {
    pub threshold: Real,
    pub duration: usize,
}

impl ThresholdDuration {
    pub fn new(threshold: Real, duration: usize) -> Self {
        Self {
            threshold,
            duration,
        }
    }
}

#[derive(Default, Clone)]
pub struct ThresholdDetector {
    time_till_armed: Option<usize>, // If this is None, then the detector is armed
    trigger: ThresholdDuration,
}

impl ThresholdDetector {
    pub fn new(trigger: &ThresholdDuration) -> Self {
        Self {
            trigger: trigger.clone(),
            ..Default::default()
        }
    }
}

pub type ThresholdEvent = Event<Real, Data>;

impl Detector for ThresholdDetector {
    type TimeType = Real;
    type ValueType = Real;
    type DataType = Data;

    fn signal(&mut self, time: Real, value: Real) -> Option<ThresholdEvent> {
        self.time_till_armed = self.time_till_armed.and_then(|time| match time {
            0 => None,
            pos => Some(pos - 1),
        });
        if self.time_till_armed.is_none() && value > self.trigger.threshold {
            self.time_till_armed = Some(self.trigger.duration);
            Some(Data {}.make_event(time))
        } else {
            None
        }
    }
}

#[derive(Default, Clone)]
pub struct ThresholdAssembler {}

impl Assembler for ThresholdAssembler {
    type DetectorType = ThresholdDetector;

    fn assemble_pulses(&mut self, source: Event<Real, Data>) -> Option<Pulse> {
        Some(Pulse {
            start: TimeValueOptional {
                time: Some(source.get_time()),
                ..Default::default()
            },
            ..Default::default()
        })
    }
}
