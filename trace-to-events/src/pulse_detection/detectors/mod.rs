pub mod advanced_muon_detector;
pub mod differential_threshold_detector;
pub mod threshold_detector;

use super::{EventData, EventPoint, Pulse, Real, RealArray, TracePoint, pulse::TimeValue};

pub(crate) trait Detector: Default + Clone {
    type TracePointType: TracePoint;
    type EventPointType: EventPoint<TimeType = <Self::TracePointType as TracePoint>::Time>;

    fn signal(
        &mut self,
        time: <Self::TracePointType as TracePoint>::Time,
        value: <Self::TracePointType as TracePoint>::Value,
    ) -> Option<Self::EventPointType>;

    fn finish(&mut self) -> Option<Self::EventPointType>;
}

pub(crate) trait Assembler: Default + Clone {
    type DetectorType: Detector;

    fn assemble_pulses(
        &mut self,
        source: <Self::DetectorType as Detector>::EventPointType,
    ) -> Option<Pulse>;
}
