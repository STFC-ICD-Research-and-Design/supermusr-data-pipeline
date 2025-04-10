use std::ops::Deref;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_pl72_run_start_generated::RunStart, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::nexus::NexusMessageHandler;

use super::{ChunkSizeSettings, NexusConfiguration, NexusDateTime, RunParameters};

// This module contains stucts used to pass messages to the `nexus_structure`` module

/// As Sample Environment Logs can be delivered via both f144 or se00 type messages,
/// a wrapper enum is required to handle them.
#[derive(Debug)]
pub(crate) enum SampleEnvironmentLog<'a> {
    LogData(f144_LogData<'a>),
    SampleEnvironmentData(se00_SampleEnvironmentData<'a>),
}

/// This is used to initialise the fields which are initialised by RunParameters or NexusConfiguration
pub(crate) struct InitialiseNewNexusStructure<'a> {
    pub(crate) parameters: &'a RunParameters,
    pub(crate) configuration: &'a NexusConfiguration,
}

/// Used to tell `nexus_structure` to initialise fields based on values in `RunParameters`
pub(crate) struct InitialiseNewNexusRun<'a> {
    pub(crate) parameters: &'a RunParameters,
}

/// Used to tell `nexus_structure` to process a `RunStart` message
/// This is used to insert any data not covered by the `InitialiseNewNexusRun` message
pub(crate) struct PushRunStart<'a>(pub(crate) RunStart<'a>);

/// Used to tell `nexus_structure` to input values from a new `FrameEventList`
/// Note this does not handle values in the `Period` hdf5 group.
pub(crate) struct PushFrameEventList<'a> {
    pub(crate) message: &'a FrameAssembledEventListMessage<'a>,
}

/// Used to tell `nexus_structure` to update the periods list in the `Periods` hdf5 group.
pub(crate) struct UpdatePeriodList<'a> {
    pub(crate) periods: &'a [u64],
}

/// Generic message used to tell `nexus_structure` a new log has been received.
pub(crate) struct PushLog<'a, T> {
    pub(crate) message: T,
    pub(crate) origin: &'a NexusDateTime,
    pub(crate) settings: &'a ChunkSizeSettings,
}

impl<T> Deref for PushLog<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.message
    }
}

/// Used to tell `nexus_structure` a new RunLog has been received.
pub(crate) type PushRunLog<'a> = PushLog<'a, &'a f144_LogData<'a>>;

/// Used to tell `nexus_structure` a new `SampleEnvironmentLog` has been received.
pub(crate) type PushSampleEnvironmentLog<'a> = PushLog<'a, &'a SampleEnvironmentLog<'a>>;

/// Used to tell `nexus_structure` a new `Alarm` has been received.
pub(crate) type PushAlarm<'a> = PushLog<'a, &'a Alarm<'a>>;

/// Enum for internally generated logs
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

/// Used to tell `nexus_structure` an internal warning has been generated
pub(crate) type PushInternallyGeneratedLogWarning<'a> = PushLog<'a, InternallyGeneratedLog<'a>>;

/// Used to tell `nexus_structure` to set the `end_time` hdf5 dataset.
pub(crate) struct SetEndTime<'a> {
    pub(crate) end_time: &'a NexusDateTime,
}

/// This trait ensures anything implementing NexusFileInterface must implement the correct `NexusMessageHandler`
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
