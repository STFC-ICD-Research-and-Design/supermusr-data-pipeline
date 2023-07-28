use std::fmt::Display;

use common::Intensity;
use super::event::{TimeValue,EventClass,SingleEvent,FuzzyReal};
use crate::window::Window;
use crate::window::smoothing_window::Stats;
use crate::{Detector, Real, SmoothingWindow, RealArray};

#[derive(Default,Debug,Clone)]
pub enum Class { #[default]Flat, Rising, Falling, LocalMax(Intensity), LocalMin(Intensity) }
impl EventClass for Class {}

impl Display for Class {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self {
                Self::Rising => 1i32,
                Self::Flat => 0i32,
                Self::Falling => -1i32,
                Self::LocalMax(value) => *value as i32,
                Self::LocalMin(value) => -(*value as i32),
            }
        ))
    }
}

const N : usize = 2;

#[derive(Default,PartialEq,Clone,Copy)]
enum SignalState { #[default]Flat, High, Low, }
impl SignalState {
    fn from_stats(Stats{value, mean, variance} : &Stats, threshold : Real) -> Option<(Self,Real)> {
        if *variance == 0. {
            return None;
        }
        let normalised = (value - mean)/variance.sqrt();
        if normalised.abs() > threshold {
            if normalised.is_sign_positive() {
                Some((SignalState::High,normalised))
            } else {
                Some((SignalState::Low,normalised))
            }
        } else {
            Some((SignalState::Flat,0.))
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
    type EventType = SingleEvent<Class>;

    fn signal(&mut self, time : Real, value: Self::ValueType) -> Option<SingleEvent<Class>> {
        let mut change_detected = false;
        for i in 0..N {
            let (current_state,normalised) = SignalState::from_stats(&value[i], self.threshold[i]).unwrap();
            if current_state != self.curr_state[i].0 {
                change_detected = true;
                self.prev_prev_state[i] = self.prev_state[i];
                self.prev_state[i] = self.curr_state[i];
                self.curr_state[i] = (current_state,
                    TimeValue::new(
                        FuzzyReal::from_real(time as Real),
                        FuzzyReal::from_real(normalised)
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

