use crate::{
    hdf5_handlers::{NexusHDF5Error, NexusHDF5Result},
    nexus::{LogMessage, LogWithOrigin, NexusMessageHandler},
};

use super::{
    AlarmChunkSize, ChunkSizeSettings, NexusConfiguration, NexusDateTime, NexusSettings,
    RunLogChunkSize, RunParameters,
};
use hdf5::types::TypeDescriptor;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
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
    SampleEnvironmentData(se00_SampleEnvironmentData<'a>),
}

///
pub(crate) struct InitialiseNewNexusStructure<'a>(
    pub(crate) &'a RunParameters,
    pub(crate) &'a NexusConfiguration,
);

pub(crate) struct InitialiseNewNexusRun<'a>(pub(crate) &'a RunParameters);

pub(crate) struct PushFrameEventList<'a>(pub(crate) &'a FrameAssembledEventListMessage<'a>);

pub(crate) struct PushRunStart<'a>(pub(crate) RunStart<'a>);

pub(crate) struct PushRunStop<'a>(pub(crate) RunStop<'a>);

pub(crate) struct PushRunLog<'a>(
    pub(crate) &'a LogWithOrigin<'a, f144_LogData<'a>>,
    pub(crate) &'a ChunkSizeSettings,
);

pub(crate) struct PushSampleEnvironmentLog<'a>(
    pub(crate) &'a LogWithOrigin<'a, SampleEnvironmentLog<'a>>,
    pub(crate) &'a ChunkSizeSettings,
);

pub(crate) type ValueLogSettings = (TypeDescriptor, AlarmChunkSize, RunLogChunkSize);

impl<'a> PushSampleEnvironmentLog<'a> {
    pub(crate) fn get_selog(&self) -> &SampleEnvironmentLog<'a> {
        self.0
    }

    pub(crate) fn get_value_log_message(&self) -> &LogWithOrigin<'a, SampleEnvironmentLog<'a>> {
        self.0
    }

    pub(crate) fn get_value_log_settings(&self) -> NexusHDF5Result<ValueLogSettings> {
        Ok((self.0.get_type_descriptor()?, self.1.alarm, self.1.runlog))
    }
}

pub(crate) struct PushAlarm<'a>(
    pub(crate) &'a LogWithOrigin<'a, Alarm<'a>>,
    pub(crate) &'a ChunkSizeSettings,
);


impl<'a> PushAlarm<'a> {
    /*pub(crate) fn get_alarm(&self) -> &SampleEnvironmentLog<'a> {
        self.0
    }

    pub(crate) fn get_value_log_message(&self) -> &LogWithOrigin<'a, Alarm<'a>> {
        self.0
    }

    pub(crate) fn get_value_log_settings(&self) -> NexusHDF5Result<ValueLogSettings> {
        Ok((self.0.get_type_descriptor()?, self.1.alarm, self.1.runlog))
    }*/
}

pub(crate) struct PushRunResumeWarning<'a>(
    pub(crate) &'a NexusDateTime,
    pub(crate) &'a NexusDateTime,
    pub(crate) &'a ChunkSizeSettings,
);

pub(crate) struct PushIncompleteFrameWarning<'a>(
    pub(crate) &'a FrameAssembledEventListMessage<'a>,
    pub(crate) &'a ChunkSizeSettings,
);

pub(crate) struct PushAbortRunWarning<'a>(
    pub(crate) i64,
    pub(crate) &'a NexusDateTime,
    pub(crate) &'a ChunkSizeSettings,
);

pub(crate) struct SetEndTime<'a>(pub(crate) &'a NexusDateTime);

pub(crate) trait HandlesAllNexusMessages:
    for<'a> NexusMessageHandler<InitialiseNewNexusStructure<'a>>
    + for<'a> NexusMessageHandler<PushFrameEventList<'a>>
    + for<'a> NexusMessageHandler<PushRunLog<'a>>
    + for<'a> NexusMessageHandler<PushRunStart<'a>>
    + for<'a> NexusMessageHandler<PushRunStop<'a>>
    + for<'a> NexusMessageHandler<PushSampleEnvironmentLog<'a>>
    + for<'a> NexusMessageHandler<PushAbortRunWarning<'a>>
    + for<'a> NexusMessageHandler<PushRunResumeWarning<'a>>
    + for<'a> NexusMessageHandler<PushIncompleteFrameWarning<'a>>
    + for<'a> NexusMessageHandler<PushAlarm<'a>>
    + for<'a> NexusMessageHandler<SetEndTime<'a>>
{
}
