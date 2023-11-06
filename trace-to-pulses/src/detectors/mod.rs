pub mod change_detector;
//pub mod composite;
pub mod basic_muon_detector;
pub mod muon_detector;
pub mod peak_detector;
pub mod pulse_detector;
pub mod threshold_detector;

use crate::{
    events::Event,
    pulse::Pulse,
    trace_iterators::feedback::FeedbackParameter,
    tracedata::{EventData, Temporal, TraceValue},
};

pub trait FeedbackDetector: Detector {
    fn is_active(&self) -> bool;
    fn modify_parameter(
        &mut self,
        _time: Self::TimeType,
        _param: &FeedbackParameter<Self::ValueType>,
    );
}

pub trait EventValuedDetector: Detector {
    type DataValueType: EventData;

    fn on_event(
        &mut self,
        event: Event<Self::TimeType, Self::DataValueType>,
    ) -> Option<Event<Self::TimeType, Self::DataType>>;
}

pub trait Detector: Default + Clone {
    type TimeType: Temporal;
    type ValueType: TraceValue;
    type DataType: EventData;
    fn signal(
        &mut self,
        time: Self::TimeType,
        value: Self::ValueType,
    ) -> Option<Event<Self::TimeType, Self::DataType>>;
}

pub trait Assembler: Default + Clone {
    type DetectorType: Detector;
    fn assemble_pulses(
        &mut self,
        source: Event<
            <Self::DetectorType as Detector>::TimeType,
            <Self::DetectorType as Detector>::DataType,
        >,
    ) -> Option<Pulse>;
}
