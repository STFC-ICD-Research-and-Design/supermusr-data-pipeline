pub mod basic_muon_detector;
pub mod threshold_detector;

use super::{
    pulse::{TimeValue, TimeValueOptional},
    EventData, EventPoint, Pulse, Real, RealArray, TracePoint,
};

pub(crate) trait Detector: Default + Clone {
    type TracePointType: TracePoint;
    type EventPointType: EventPoint<TimeType = <Self::TracePointType as TracePoint>::TimeType>;

    fn signal(
        &mut self,
        time: <Self::TracePointType as TracePoint>::TimeType,
        value: <Self::TracePointType as TracePoint>::ValueType,
    ) -> Option<Self::EventPointType>;
}

pub(crate) trait Assembler: Default + Clone {
    type DetectorType: Detector;

    fn assemble_pulses(
        &mut self,
        source: <Self::DetectorType as Detector>::EventPointType,
    ) -> Option<Pulse>;
}
