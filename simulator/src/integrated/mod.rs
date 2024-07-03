use supermusr_common::{FrameNumber, Time};

pub(crate) struct Action {
    class: ActionClass,
    min_pause_ms: Time,
}

pub(crate) enum ActionClass {
    RunStart {
        name: String,
        instrument: String,
    },
    RunStop {
        name: String,
    },
    EmitFrames {
        start: FrameNumber,
        source: FrameSource,
        delay_ms: Time,
        end: FrameNumber,
    },
}

pub(crate) enum TraceSource {
}

pub(crate) enum FrameSource {
    AggregatedFrame {
        num_channels: usize,
        source: TraceSource
    },
    AutoDigitisers {},
    DigitiserList {}
}

pub(crate) struct Simulation {
    pub(crate) trace_sources: Vec<TraceSource>,
    pub(crate) schedule: Vec<Action>,
}
