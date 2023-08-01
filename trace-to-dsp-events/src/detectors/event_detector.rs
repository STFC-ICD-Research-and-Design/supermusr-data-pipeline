use std::fmt::Display;

use common::Intensity;
use crate::events::{TimeValue,EventData,SimpleEvent};
use crate::window::smoothing_window::{Stats, SNRSign};
use crate::{Detector, Real};

#[derive(Default,Debug,Clone)]
pub enum Class { #[default]Flat, Rising, Falling, LocalMax, LocalMin }
#[derive(Default,Debug,Clone)]
pub struct Data {
    pub(super) class : Class,
    peak_intensity : Option<Real>,
    area_under_curve : Option<Real>,
    half_peak_full_width : Option<Real>,
    start : Option<Real>,
    end : Option<Real>,
}


impl EventData for Data {}
impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self.class {
                Class::Rising => 1i32,
                Class::Flat => 0i32,
                Class::Falling => -1i32,
                Class::LocalMax => self.peak_intensity.unwrap_or_default() as i32,
                Class::LocalMin => -(self.peak_intensity.unwrap_or_default() as i32),
            }
        ))
    }
}

const N : usize = 2;

#[derive(Default,PartialEq,Clone,Copy)]
enum SignalState { #[default]Flat, High, Low, }
impl SignalState {
    fn from_stats(stats : &Stats, threshold : Real) -> Option<(Self,Real)> {
        if stats.variance == 0. {
            return None;
        }
        match stats.signal_over_noise_sign(threshold) {
            SNRSign::Pos => Some((SignalState::High,stats.get_normalized_value())),
            SNRSign::Neg => Some((SignalState::Low,stats.get_normalized_value())),
            SNRSign::Zero => Some((SignalState::Flat,0.)),
        }
    }
}

#[derive(Default)]
pub struct EventsDetector {
    prev_prev_state : [(SignalState, TimeValue); N],
    prev_state : [(SignalState, TimeValue); N],
    curr_state : [(SignalState, TimeValue); N],

    // Parameters
    threshold: [Real;N],
}

impl EventsDetector {
    pub fn new(threshold: [Real;2]) -> EventsDetector {
        EventsDetector {
            threshold,
            ..Default::default()
        }
    }
}
impl Detector for EventsDetector {
    type TimeType = Real;
    type ValueType = [Stats;2];
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time : Real, value: Self::ValueType) -> Option<SimpleEvent<Data>> {
        let mut change_detected = false;
        for i in 0..N {
            let (current_state,normalised) = SignalState::from_stats(&value[i], self.threshold[i]).unwrap();
            if current_state != self.curr_state[i].0 {
                change_detected = true;
                self.prev_prev_state[i] = self.prev_state[i];
                self.prev_state[i] = self.curr_state[i];
                self.curr_state[i] = (current_state,
                    TimeValue::new(
                        Real::from(time as Real),
                        Real::from(normalised)
                    )
                );
            }
        }
        if change_detected {
            match (self.prev_prev_state[1].0,self.prev_state[1].0,self.curr_state[1].0) {
                (SignalState::Low,SignalState::Flat,SignalState::High) => None,
                (SignalState::High,SignalState::Flat,SignalState::Low) => None,
                (SignalState::High,SignalState::Flat,SignalState::High) => None,
                (SignalState::Low,SignalState::Flat,SignalState::Low) => None,
                _ => None,
            }
        }
        else {
            None
        }
    }
}

