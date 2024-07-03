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
        end: FrameNumber,
    },
}

pub(crate) struct Simulation {
    pub(crate) schedule: Vec<Action>,
}
