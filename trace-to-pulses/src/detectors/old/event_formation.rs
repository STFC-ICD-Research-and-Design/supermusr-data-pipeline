use std::fmt::Display;

use super::{change_detector, event_detector};
use crate::{
    events::{Event, EventData, EventWithData, SimpleEvent, TimeValue},
    window::smoothing_window::{SNRSign, Stats},
    Detector, Real,
};
use common::Intensity;

#[derive(Default, Debug, Clone)]
pub enum Class {
    #[default]
    Pulse,
}
#[derive(Default, Debug, Clone)]
pub struct Data {
    peak_intensity: Option<Real>,
    area_under_curve: Option<Real>,
    half_peak_full_width: Option<Real>,
    start: Option<Real>,
    end: Option<Real>,
}

impl EventData for Data {}
impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{},{},{},{},{}",
            self.peak_intensity.unwrap_or(-1.),
            self.area_under_curve.unwrap_or(-1.),
            self.half_peak_full_width.unwrap_or(-1.),
            self.start.unwrap_or(-1.),
            self.end.unwrap_or(-1.),
        ))?;
        Ok(())
    }
}

#[derive(Default)]
pub struct EventFormer<const N: usize> {
    states: Vec<[Box<dyn EventWithData<DataType = change_detector::Data>>; N]>,
}

impl<const N: usize> EventFormer<N> {
    pub fn new() -> EventFormer<N> {
        EventFormer::default()
    }
}
impl<const N: usize> Detector for EventFormer<N> {
    type TimeType = Real;
    type ValueType = [Stats; 2];
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<SimpleEvent<Data>> {
        match self.states[0][0].get_data().class {
            change_detector::Class::Flat => match self.states[1][0].get_data().class {
                change_detector::Class::Flat => todo!(),
                change_detector::Class::Rising => todo!(),
                change_detector::Class::Falling => todo!(),
            },
            change_detector::Class::Rising => todo!(),
            change_detector::Class::Falling => todo!(),
        }
    }
}
