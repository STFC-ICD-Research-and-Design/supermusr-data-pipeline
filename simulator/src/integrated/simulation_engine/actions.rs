use crate::integrated::simulation_elements::{
    run_messages::{SendAlarm, SendRunLogData, SendRunStart, SendRunStop, SendSampleEnvLog},
    utils::IntConstant,
    Interval,
};
use serde::Deserialize;
use tracing::error;

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum SelectionModeOptions {
    PopFront,
    ReplaceRandom,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "source")]
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

impl SendAggregatedEventListOptions {
    pub(crate) fn validate(&self, num_channels: usize) -> bool {
        if let Some(upper) = self.channel_indices.range_inclusive().last() {
            if upper >= num_channels {
                error!("Aggregated Event List channel index too large {upper} >= {num_channels}");
                return false;
            }
            true
        } else {
            false
        }
    }
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
    pub(crate) event_list_index: usize,
    pub(crate) repeat: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct GenerateEventList {
    pub(crate) event_list_index: usize,
    pub(crate) repeat: usize,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Loop<A> {
    pub(crate) start: IntConstant,
    pub(crate) end: IntConstant,
    pub(crate) schedule: Vec<A>,
}

impl Loop<FrameAction> {
    fn validate(&self, num_digitisers: usize, num_channels: usize) -> bool {
        if self.start.value() > self.end.value() {
            error!("Frame start index > end index");
            return false;
        }
        for action in &self.schedule {
            if !action.validate(num_digitisers, num_channels) {
                return false;
            }
        }
        true
    }
}

impl Loop<DigitiserAction> {
    fn validate(&self, num_digitisers: usize) -> bool {
        if self.start.value() > self.end.value() {
            error!("Digitiser start index > end index");
            return false;
        }
        if self.end.value() >= num_digitisers as i32 {
            error!(
                "Digitiser end index too large {0} >= {num_digitisers}",
                self.end.value()
            );
            return false;
        }
        true
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Timestamp {
    Now,
    AdvanceByMs(usize),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum TracingLevel {
    Info,
    Debug,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct TracingEvent {
    pub(crate) level: TracingLevel,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum Action {
    Comment(String),
    TracingEvent(TracingEvent),
    WaitMs(usize),
    EnsureDelayMs(usize),
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

impl Action {
    pub(crate) fn validate(&self, num_digitisers: usize, num_channels: usize) -> bool {
        match self {
            Action::FrameLoop(frame_loop) => frame_loop.validate(num_digitisers, num_channels),
            _ => true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum FrameAction {
    Comment(String),
    WaitMs(usize),
    EnsureDelayMs(usize),
    TracingEvent(TracingEvent),
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

impl FrameAction {
    fn validate(&self, num_digitisers: usize, num_channels: usize) -> bool {
        match self {
            FrameAction::SendAggregatedFrameEventList(frame_event_list) => {
                frame_event_list.validate(num_channels)
            }
            FrameAction::DigitiserLoop(digitiser_loop) => digitiser_loop.validate(num_digitisers),
            _ => true,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum DigitiserAction {
    Comment(String),
    WaitMs(usize),
    EnsureDelayMs(usize),
    TracingEvent(TracingEvent),
    //
    SendDigitiserTrace(SendTraceOptions),
    SendDigitiserEventList(SendDigitiserEventListOptions),
    //
    GenerateTrace(GenerateTrace),
    GenerateEventList(GenerateEventList),
}
