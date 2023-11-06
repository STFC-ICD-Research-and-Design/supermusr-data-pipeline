use std::array::{from_fn, from_ref};
use std::fmt::Display;

use crate::events::{
    Event,
    EventData,
    EventWithData,
    multiple_events::MultipleEvents,
    SimpleEvent,
    TimeValue
};
use crate::window::smoothing_window::{SNRSign, Stats};
use crate::window::Window;
use crate::{Detector, Real, RealArray, SmoothingWindow};


#[derive(Default, Debug, Clone, PartialEq)]
pub enum SignClass {
    #[default]
    Zero,
    Pos,
    Neg,
}

#[derive(Default, Debug, Clone)]
pub struct SignData {
    pub(super) class: SignClass,
}
impl SignData {
    pub fn get_class(&self) -> &SignClass {
        &self.class
    }
}

impl Display for SignData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{0}",
            match self.class {
                SignClass::Pos => 1i32,
                SignClass::Zero => 0i32,
                SignClass::Neg => -1i32,
            }
        ))
    }
}
impl EventData for SignData {}
