use super::{
    simulation_elements::run_messages::{
        SendAlarm, SendRunLogData, SendRunStart, SendRunStop, SendSampleEnvLog,
    },
    Interval,
};
use serde::Deserialize;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectionModeOptions {
    PopFront,
    ReplaceRandom,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SourceOptions {
    NoSource,
    SelectFromCache(SelectionModeOptions),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SendAggregatedEventListOptions {
    pub(crate) source_options: SourceOptions,
    pub(crate) channel_indices: Interval<usize>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SendDigitiserEventListOptions(pub(crate) SourceOptions);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct SendTraceOptions(pub(crate) SelectionModeOptions);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GenerateTrace {
    pub(crate) selection_mode: SelectionModeOptions,
    pub(crate) repeat: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GenerateEventList {
    pub(crate) template_index: usize,
    pub(crate) repeat: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Loop<A> {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) schedule: Vec<A>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Timestamp {
    Now,
    AdvanceByMs(usize),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Action {
    Comment(String),
    WaitMs(usize),
    SendRunStart(SendRunStart),
    SendRunStop(SendRunStop),
    SendRunLogData(SendRunLogData),
    SendSampleEnvLog(SendSampleEnvLog),
    SendAlarm(SendAlarm),
    //
    FrameLoop(Loop<FrameAction>),
    //
    SetTimestamp(Timestamp),
    SetVetoFlags(u16),
    SetPeriod(u64),
    SetProtonsPerPulse(u8),
    SetRunning(bool),
    //
    GenerateTrace(GenerateTrace),
    GenerateEventList(GenerateEventList),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FrameAction {
    Comment(String),
    WaitMs(usize),
    //
    SendAggregatedFrameEventList(SendAggregatedEventListOptions),
    //
    DigitiserLoop(Loop<DigitiserAction>),
    //
    SetTimestamp(Timestamp),
    //
    GenerateTrace(GenerateTrace),
    GenerateEventList(GenerateEventList),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DigitiserAction {
    Comment(String),
    WaitMs(usize),
    //
    SendDigitiserTrace(SendTraceOptions),
    SendDigitiserEventList(SendDigitiserEventListOptions),
    //
    GenerateTrace(GenerateTrace),
    GenerateEventList(GenerateEventList),
}
