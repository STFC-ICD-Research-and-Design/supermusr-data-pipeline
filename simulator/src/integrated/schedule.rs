use super::run_messages::{Alarm, RunLogData, RunStart, RunStop, SampleEnvLog};
use serde::Deserialize;
use supermusr_common::FrameNumber;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectionModeOptions {
    PopFront,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Source {
    pub(crate) selection_mode: SelectionModeOptions,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum LoopVariable {
    Generic,
    Frame,
    Digitiser,
    Channel,
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Action {
    WaitMs(usize),
    SendRunStart(RunStart),
    SendRunStop(RunStop),
    SendRunLogData(RunLogData),
    SendSampleEnvLog(SampleEnvLog),
    SendAlarm(Alarm),
    //
    SendDigitiserTrace(Source),
    SendDigitiserEventList(Source),
    SendAggregatedFrameEventList(Source),
    //
    Loop(Loop),
    //
    SetFrame(FrameNumber),
    SetTimestampToNow(),
    AdvanceTimestampByMs(usize),
    SetDigitiserIndex(usize),
    SetChannelIndex(usize),
    SetVetoFlags(u16),
    SetPeriod(u64),
    SetProtonsPerPulse(u8),
    SetRunning(bool),
    //
    GenerateTrace(Source),
    GenerateEventList(usize),
}
