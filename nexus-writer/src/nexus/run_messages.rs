use super::NexusDateTime;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop,
    ecs_f144_logdata_generated::f144_LogData,
    ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

/// As Sample Environment Logs can be delivered via both f144 or se00 type messages,
/// a wrapper enum is required to handle them.
pub(crate) enum SampleEnvironmentLogType {
    LogData,
    SampleEnvironmentData,
}

#[derive(Debug)]
pub(crate) enum SampleEnvironmentLog<'a> {
    LogData(f144_LogData<'a>),
    SampleEnvironmentData(se00_SampleEnvironmentData<'a>)
}

pub(crate) struct PushFrameEventList<'a>(FrameAssembledEventListMessage<'a>);

pub(crate) struct PushRunStart<'a>(RunStart<'a>);

pub(crate) struct PushRunStop<'a>(RunStop<'a>);

pub(crate) struct PushRunLogData<'a>(f144_LogData<'a>);

pub(crate) struct PushSampleEnvironmentLog<'a>(SampleEnvironmentLog<'a>, i64);

pub(crate) struct SetEndTime(NexusDateTime);