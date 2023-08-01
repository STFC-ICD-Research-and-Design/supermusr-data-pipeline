use std::fmt::Display;

use common::Intensity;
use crate::events::{TimeValue,EventData,SimpleEvent};
use crate::window::smoothing_window::{Stats, SNRSign};
use crate::{Detector, Real};
use super::{event_detector, change_detector};

#[derive(Default,Debug,Clone)]
pub enum Class { #[default]Pulse }
#[derive(Default,Debug,Clone)]
pub struct Data {
    peak_intensity : Option<Real>,
    area_under_curve : Option<Real>,
    half_peak_full_width : Option<Real>,
    start : Option<Real>,
    end : Option<Real>,
}


impl EventData for Data {}
impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{},{},{},{},{}",
            self.peak_intensity.unwrap_or(-1.),
            self.area_under_curve.unwrap_or(-1.),
            self.half_peak_full_width.unwrap_or(-1.),
            self.start.unwrap_or(-1.),
            self.end.unwrap_or(-1.),
        ))?;
        Ok(())
    }
}

const N : usize = 3;

#[derive(Default)]
pub struct EventFormer {
    prev_prev_state : [SimpleEvent<change_detector::Data>; N],
    prev_state : [SimpleEvent<change_detector::Data>; N],
    curr_state : [SimpleEvent<change_detector::Data>; N],
}

impl EventFormer {
    pub fn new() -> EventFormer {
        EventFormer::default()
    }
}
impl Detector for EventFormer {
    type TimeType = Real;
    type ValueType = [Stats;2];
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time : Real, value: Self::ValueType) -> Option<SimpleEvent<Data>> {
        match self.curr_state[0].data.class {
            change_detector::Class::Flat => match self.prev_state[0].data.class {
                change_detector::Class::Flat => todo!(),
                change_detector::Class::Rising => todo!(),
                change_detector::Class::Falling => todo!(),
            },
            change_detector::Class::Rising => todo!(),
            change_detector::Class::Falling => todo!(),
        }
    }
}

