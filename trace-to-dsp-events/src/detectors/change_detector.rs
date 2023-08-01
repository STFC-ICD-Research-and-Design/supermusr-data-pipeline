
use std::array::{from_ref, from_fn};
use std::fmt::Display;

use crate::events::{EventData,TimeValue, SimpleEvent, MultipleEvents, Event, EventWithData};
use crate::window::Window;
use crate::window::smoothing_window::{Stats, SNRSign};
use crate::{Detector, Real, SmoothingWindow, RealArray};

#[derive(Default,Debug,Clone,PartialEq)]
pub enum Class { #[default]Flat, Rising, Falling }

#[derive(Default,Debug,Clone)]
pub struct Data {
    pub(super) class : Class,
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self.class {
                Class::Rising => 1i32,
                Class::Flat => 0i32,
                Class::Falling => -1i32,
            }
        ))
    }
}
impl EventData for Data {}




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
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time : Real, value: Real) -> Option<SimpleEvent<Data>> {
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
            event_class.map(|e|SimpleEvent::new(now.time, Data{ class: e.clone() }))
        } else {
            self.prev = Some(value);
            None
        }
    }
}



#[derive(Default,Debug,Clone,PartialEq)]
pub enum SignClass { #[default]Zero, Pos, Neg }

#[derive(Default,Debug,Clone)]
pub struct SignData {
    pub(super) class : SignClass,
}
impl SignData {
    pub fn get_class(&self) -> &SignClass { &self.class }
}

impl Display for SignData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{0}",
            match self.class {
                SignClass::Pos => 1i32,
                SignClass::Zero => 0i32,
                SignClass::Neg => -1i32,
            }
        ))
    }
}
impl EventData for SignData {}



#[derive(Default)]
pub struct SignDetector {
    mode : Option<SignClass>,
    threshold: Real,
}
impl SignDetector {
    pub fn new(threshold: Real) -> Self {Self {
        threshold,
        ..Default::default()
    }}
}
impl Detector for SignDetector {
    type TimeType = Real;
    type ValueType = Real;
    type EventType = SimpleEvent<SignData>;

    fn signal(&mut self, time : Real, value: Real) -> Option<SimpleEvent<SignData>> {
        let now = TimeValue::new(Real::from(time as Real), Real::from(value));
        let new_mode = Some(
            if value.abs() <= self.threshold {
                SignClass::Zero
            } else if value > 0. {
                SignClass::Pos
            } else {
                SignClass::Neg
            }
        );

        if new_mode == self.mode {
            None
        } else {
            self.mode = new_mode;
            self.mode.clone().map(|e|SimpleEvent::new(now.time, SignData{ class: e }))
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
    type EventType = SimpleEvent<Data>;

    fn signal(&mut self, time : Real, value: Real) -> Option<SimpleEvent<Data>> {
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
            
            let event = event_class.map(|e|SimpleEvent::new(now.time,Data{ class: e }));
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