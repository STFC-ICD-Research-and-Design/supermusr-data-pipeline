use std::path::Path;

use super::{NexusConfiguration, NexusDateTime, NexusSettings, RunParameters};
use hdf5::File;
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

pub(crate) struct InitialiseNewNexusStructure<'a>(pub(crate) &'a RunParameters, pub(crate) &'a NexusConfiguration);

pub(crate) struct InitialiseNewNexusRun<'a>(pub(crate) &'a RunParameters);

pub(crate) struct PushFrameEventList<'a>(pub(crate) FrameAssembledEventListMessage<'a>);

pub(crate) struct PushRunStart<'a>(pub(crate) RunStart<'a>);

pub(crate) struct PushRunStop<'a>(pub(crate) RunStop<'a>);

pub(crate) struct PushRunLogData<'a>(pub(crate) f144_LogData<'a>);

pub(crate) struct PushSampleEnvironmentLog<'a>(pub(crate) SampleEnvironmentLog<'a>, i64);

pub(crate) struct PushRunResumeWarning<'a>(pub(crate) &'a NexusDateTime, pub(crate) i64, pub(crate) &'a NexusSettings);

pub(crate) struct PushIncompleteFrameWarning<'a>(pub(crate) &'a FrameAssembledEventListMessage<'a>, pub(crate) i64, pub(crate) &'a NexusSettings);

pub(crate) struct PushAbortRunWarning<'a>(pub(crate) i64, pub(crate) &'a NexusDateTime, pub(crate) &'a NexusSettings);

pub(crate) struct SetEndTime<'a>(pub(crate) &'a NexusDateTime);