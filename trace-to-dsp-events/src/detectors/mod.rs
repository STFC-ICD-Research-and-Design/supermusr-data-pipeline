pub mod peak_detector;
pub mod event_detector;
pub mod change_detector;
pub mod event_formation;
pub mod composite;

use crate::events::Event;

pub trait Detector {

    type TimeType;
    type ValueType;
    type EventType : Event;
    fn signal(&mut self, time : Self::TimeType, value: Self::ValueType) -> Option<Self::EventType>;
}