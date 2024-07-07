use super::run_messages::{Alarm, RunLogData, RunStart, RunStop, SampleEnvLog};
use serde::Deserialize;
use supermusr_common::{Channel, DigitizerId, FrameNumber, Time};
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectionModeOptions {
    PopFront
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Source {
    pub(crate) selection_mode : SelectionModeOptions,
}


#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LoopVariable {
    Generic, Frame, Digitiser, Channel
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Loop {
    pub(crate) variable: LoopVariable,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) schedule: Vec<Action>,
}

impl Loop {
    fn validate(&self) -> bool {
        for action in &self.schedule {
            if let Action::Loop(lp) = action {
                if !lp.validate() {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Action {
    WaitMs(usize),
    RunStart(RunStart),
    RunStop(RunStop),
    RunLogData(RunLogData),
    SampleEnvLog(SampleEnvLog),
    Alarm(Alarm),
    //
    DigitiserTrace(),
    DigitiserEventList(),
    AggregatedFrameEventList(),
    //
    EmitFrameEventList(),
    EmitDigitiserEventList(),
    EmitDigitiserTrace(),
    Loop(Loop),
    //
    SetFrame(FrameNumber),
    SetVetoFlags(u16),
    SetPeriod(u64),
    SetProtonsPerPulse(u8),
    SetRunning(bool),
    //
    GenerateTrace(Source),
    GenerateEventList(usize),
}