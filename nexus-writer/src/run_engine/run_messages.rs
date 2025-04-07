use crate::nexus::{LogWithOrigin, NexusMessageHandler};

use super::{ChunkSizeSettings, NexusConfiguration, NexusDateTime, RunParameters};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_pl72_run_start_generated::RunStart, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

/// As Sample Environment Logs can be delivered via both f144 or se00 type messages,
/// a wrapper enum is required to handle them.
#[derive(Debug)]
pub(crate) enum SampleEnvironmentLog<'a> {
    LogData(f144_LogData<'a>),
    SampleEnvironmentData(se00_SampleEnvironmentData<'a>),
}

/// This is used to initialise the fields which are initialised by RunParameters or NexusConfiguration
pub(crate) struct InitialiseNewNexusStructure<'a>(
    pub(crate) &'a RunParameters,
    pub(crate) &'a NexusConfiguration,
);

pub(crate) struct InitialiseNewNexusRun<'a>(pub(crate) &'a RunParameters);

pub(crate) struct PushFrameEventList<'a> {
    pub(crate) message: &'a FrameAssembledEventListMessage<'a>,
}

pub(crate) struct UpdatePeriodList<'a> {
    pub(crate) periods: &'a [u64],
}

pub(crate) struct PushRunStart<'a>(pub(crate) RunStart<'a>);

pub(crate) struct PushRunLog<'a> {
    pub(crate) runlog: &'a LogWithOrigin<'a, f144_LogData<'a>>,
    pub(crate) settings: &'a ChunkSizeSettings,
}

pub(crate) struct PushSampleEnvironmentLog<'a> {
    pub(crate) selog: &'a LogWithOrigin<'a, SampleEnvironmentLog<'a>>,
    pub(crate) settings: &'a ChunkSizeSettings,
}

pub(crate) struct PushAlarm<'a>(
    pub(crate) &'a LogWithOrigin<'a, Alarm<'a>>,
    pub(crate) &'a ChunkSizeSettings,
);

pub(crate) enum InternallyGeneratedLog<'a> {
    RunResume {
        resume_time: &'a NexusDateTime,
    },
    IncompleteFrame {
        frame: &'a FrameAssembledEventListMessage<'a>,
    },
    AbortRun {
        stop_time_ms: i64,
    },
}
pub(crate) struct PushInternallyGeneratedLogWarning<'a> {
    pub(crate) message: InternallyGeneratedLog<'a>,
    pub(crate) origin: &'a NexusDateTime,
    pub(crate) settings: &'a ChunkSizeSettings,
}

pub(crate) struct SetEndTime<'a> {
    pub(crate) end_time: &'a NexusDateTime,
}

pub(crate) trait HandlesAllNexusMessages:
    for<'a> NexusMessageHandler<InitialiseNewNexusStructure<'a>>
    + for<'a> NexusMessageHandler<PushFrameEventList<'a>>
    + for<'a> NexusMessageHandler<UpdatePeriodList<'a>>
    + for<'a> NexusMessageHandler<PushRunLog<'a>>
    + for<'a> NexusMessageHandler<PushRunStart<'a>>
    + for<'a> NexusMessageHandler<PushSampleEnvironmentLog<'a>>
    + for<'a> NexusMessageHandler<PushInternallyGeneratedLogWarning<'a>>
    + for<'a> NexusMessageHandler<PushAlarm<'a>>
    + for<'a> NexusMessageHandler<SetEndTime<'a>>
{
}
