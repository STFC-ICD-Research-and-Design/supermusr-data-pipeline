use std::collections::VecDeque;
use std::fmt::Display;

use crate::events::{Event, EventData, EventWithData, SimpleEvent, TimeValue};
use crate::window::smoothing_window::{SNRSign, Stats};
use crate::{Detector, Real, RealArray};
use common::Intensity;
use fitting::approx::assert_abs_diff_eq;
use fitting::Gaussian;
use fitting::ndarray::{array, Array, Array1};
use num::complex::ComplexFloat;

use super::change_detector;
use super::composite::CompositeDetector;

#[derive(Default, Debug, Clone)]
pub enum Class {
    #[default]
    Flat,
    Rising,
    Falling,
    LocalMax,
    LocalMin,
}
#[derive(Default, Debug, Clone)]
pub struct Data {
    pub(super) class: Class,
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
            "{0}",
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

#[derive(Default, PartialEq, Clone, Copy)]
enum SignalState {
    #[default]
    Flat,
    High,
    Low,
}
impl SignalState {
    fn from_stats(stats: &Stats, threshold: Real) -> Option<(Self, Real)> {
        if stats.variance == 0. {
            return None;
        }
        match stats.signal_over_noise_sign(threshold) {
            SNRSign::Pos => Some((SignalState::High, stats.get_normalized_value())),
            SNRSign::Neg => Some((SignalState::Low, stats.get_normalized_value())),
            SNRSign::Zero => Some((SignalState::Flat, 0.)),
        }
    }
}

enum EventsDetectorState {
    WaitingForNonzero,
    WaitingForChange,
}
pub struct EventsDetector<const N: usize> {
    state : EventsDetectorState,

    prev_signal: VecDeque<(Real, Real)>,
    pulses_begun: VecDeque<Real>,
    pulses_found: VecDeque<Gaussian<Real>>,

    change_detector: CompositeDetector<N, SimpleEvent<change_detector::Data>>,
}

impl<const N: usize> EventsDetector<N> {
    pub fn new(
        change_detector: CompositeDetector<N, SimpleEvent<change_detector::Data>>,
    ) -> EventsDetector<N> {
        EventsDetector {
            change_detector,
            state: EventsDetectorState::WaitingForNonzero,
            prev_signal: VecDeque::<(Real, Real)>::default(),
            pulses_begun: VecDeque::<Real>::default(),
            pulses_found: VecDeque::<Gaussian<Real>>::default(),
        }
    }
    fn extract_gaussian(&mut self) -> Gaussian<Real> {
        let x_vec : Array1<Real> = self.prev_signal.iter().map(|s|s.0).collect();
        let y_vec : Array1<Real> = self.prev_signal.iter().map(|s|s.1).collect();
        
        let estimated = Gaussian::fit(x_vec, y_vec).unwrap_or_else(|e| match e {
            fitting::gaussian::GaussianError::GivenYVecContainsNegativeValue => panic!("{}"),
            fitting::gaussian::GaussianError::GivenXVecHasNoElement => panic!(),
            fitting::gaussian::GaussianError::Linalg(la) => match la {
                fitting::linalg::LinalgError::EquationsHaveNoSolutions => panic!(),
                fitting::linalg::LinalgError::EquationsHaveInfSolutions => panic!(),
            },
        });

        self.prev_signal.clear();
        for (time, values) in &mut self.prev_signal {
            *values -= estimated.value(*time);
        }
        estimated
    }
}
impl<const N: usize> Detector for EventsDetector<N> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time: Real, value: Self::ValueType) -> Option<SimpleEvent<Data>> {
        self.prev_signal.push_back((time, value[0] - self.pulses_found.iter().map(|g|g.value(time)).sum::<Real>()));
        match self.change_detector.signal(time, value) {
            Some(events) => {
                match &self.state {
                    EventsDetectorState::WaitingForNonzero => {
                        let mut iter = events.into_iter();
                        if let Some(event) = iter.find(|e| e.get_data().get_index() == 1) {
                            match event.get_data().get_data().class {
                                change_detector::Class::Flat => {
                                }
                                change_detector::Class::Rising => {
                                    let new_gaussian = self.extract_gaussian();
                                    self.pulses_found.push_back(new_gaussian);
                                    self.state = EventsDetectorState::WaitingForChange
                                },
                                change_detector::Class::Falling => {
                                    let new_gaussian = self.extract_gaussian();
                                    self.pulses_found.push_back(new_gaussian);
                                    self.state = EventsDetectorState::WaitingForChange
                                },
                            }
                        }
                    },
                    EventsDetectorState::WaitingForChange => {
                        let mut iter = events.into_iter();
                        if let Some(event) = iter.find(|e| e.get_data().get_index() == 1) {
                            let new_gaussian = self.extract_gaussian();
                            self.pulses_found.push_back(new_gaussian);
                            self.state = EventsDetectorState::WaitingForChange
                        }
                    },
                }
            },
            None => {},
        };
        if let Some(pulse) = self.pulses_found.front() {
            if pulse.value(time).abs() < 1e-6 {
                let pulse = self.pulses_found.pop_front().unwrap();
                return Some(
                    SimpleEvent::<Data>::new(
                        *pulse.mu(),
                         Data {
                             class: Class::Flat,
                             peak_intensity: Some(*pulse.a()),
                             area_under_curve: None,
                             half_peak_full_width: Some(*pulse.sigma()),
                             start: None,
                             end: None
                        }
                    )
                );
            }
        }
        None
    }
}
