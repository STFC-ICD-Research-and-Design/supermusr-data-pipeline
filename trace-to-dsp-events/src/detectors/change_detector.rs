
use std::array::{from_ref, from_fn};
use std::fmt::Display;

use super::event::{EventClass,TimeValue, SimpleEvent, MultipleEvents};
use crate::window::Window;
use crate::window::smoothing_window::{Stats, SNRSign};
use crate::{Detector, Real, SmoothingWindow, RealArray};

#[derive(Default,Debug,Clone,PartialEq)]
pub enum Class { #[default]Flat, Rising, Falling }

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
impl EventClass for Class {}




#[derive(Default)]
pub struct SimpleChangeDetector {
    mode : Class,
    prev : Option<Real>,
    threshold: Real,
}
impl SimpleChangeDetector {
    pub fn new(threshold: Real) -> Self {Self {
        threshold,
        ..Default::default()
    }}
}
impl Detector for SimpleChangeDetector {
    type TimeType = Real;
    type ValueType = Real;
    type EventType = SimpleEvent<Class>;

    fn signal(&mut self, time : Real, value: Real) -> Option<SimpleEvent<Class>> {
        if let Some(prev_value) = self.prev
        {
            let now = TimeValue::new(Real::from(time as Real), Real::from(value));
            let new_mode = {
                if (value - prev_value).abs() <= self.threshold {
                    Class::Flat
                } else if value > prev_value {
                    Class::Rising
                } else {
                    Class::Falling
                }
            };

            let event_class = if new_mode == self.mode {
                None
            } else {
                Some(new_mode.clone())
            };
            self.mode = new_mode;
            self.prev = Some(value);
            event_class.map(|e|SimpleEvent::new(e.clone(),now))
        } else {
            self.prev = Some(value);
            None
        }
    }
}






#[derive(Default)]
pub struct ChangeDetector {
    mode : Class,
    start : TimeValue,
    peak : TimeValue,
    trough : TimeValue,

    threshold: Real,
    _influence: Real,
    window: SmoothingWindow,
}
impl ChangeDetector {
    pub fn new(lag: usize, threshold: Real, _influence: Real) -> ChangeDetector {
        ChangeDetector {
            threshold,
            _influence,
            window: SmoothingWindow::new(lag),
            ..Default::default()
        }
    }
}
impl Detector for ChangeDetector {
    type TimeType = Real;
    type ValueType = Real;
    type EventType = SimpleEvent<Class>;

    fn signal(&mut self, time : Real, value: Real) -> Option<SimpleEvent<Class>> {
        if let Some(stats) = self.window.stats()
        {
            if self.peak.value < value {
                self.peak = TimeValue::from_exact(time, value);
            }
            if self.trough.value > value {
                self.trough = TimeValue::from_exact(time, value);
            }
            let now = TimeValue::new(Real::from(time as Real), Real::from(value));
            let new_mode = match stats.signal_over_noise_sign(self.threshold) {
                SNRSign::Pos => Class::Rising,
                SNRSign::Neg => Class::Falling,
                SNRSign::Zero => {
                    self.window.push(value);
                    Class::Flat
                }
            };

            let event_class = if new_mode == self.mode {
                None
            } else {
                Some(new_mode.clone())
            };
            
            let event = event_class.map(|e|SimpleEvent::new(e.clone(),now.clone()));
            /*    match e {
                    Class::Rising => self.peak,
                    Class::Falling => self.trough,
                    Class::Flat => self.peak,
                }
            )); */
            self.mode = new_mode;
            //self.start = now.clone();
            event
        } else {
            self.window.push(value);
            self.peak = TimeValue::from_exact(time, value);
            self.trough = TimeValue::from_exact(time, value);
            None
        }
    }
}






//#[derive(Default)]
pub struct FiniteDifferenceChangeDetector<const N : usize> {
    detectors: [SimpleChangeDetector;N],
}
    
impl<const N : usize> FiniteDifferenceChangeDetector<N> {
    pub fn new(detectors: [SimpleChangeDetector;N]) -> FiniteDifferenceChangeDetector<N> {
        FiniteDifferenceChangeDetector::<N> {
            detectors,
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq)]
pub struct FDClass {
    index : usize,
    class : Class
}
impl EventClass for FDClass {}

impl Display for FDClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}:{1}",self.index,self.class))
    }
}

impl<const N : usize> Detector for FiniteDifferenceChangeDetector<N> {
    type TimeType = Real;
    type ValueType = RealArray<N>;
    type EventType = MultipleEvents<SimpleEvent<FDClass>>;

    fn signal(&mut self, time : Real, value: RealArray<N>) -> Option<Self::EventType> {
        let events : Vec<SimpleEvent<FDClass>> = self.detectors
            .iter_mut()
            .enumerate()
            .map(|(i,detector)| {
                let event = detector.signal(time,value[i]).map(|v| (i,v))?;
                Some(SimpleEvent::new(FDClass{index: event.0, class: event.1.class},event.1.time_value))
            })
            .flatten()
            .collect();
        if events.is_empty() {
            None
        } else {
            Some(MultipleEvents::new(events))
        }
    }
}