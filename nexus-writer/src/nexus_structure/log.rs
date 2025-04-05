use std::ops::Deref;

use hdf5::{
    dataset::Chunk,
    types::{TypeDescriptor, VarLenUnicode},
    Dataset, Group,
};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    hdf5_handlers::{GroupExt, NexusHDF5Result},
    nexus::{nexus_class, AlarmMessage, LogMessage, LogWithOrigin, NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::{PushAlarm, PushSampleEnvironmentLog, ValueLogSettings}, AlarmChunkSize, RunLogChunkSize, SampleEnvironmentLog},
};

pub(crate) struct Log {
    type_descriptor: Option<TypeDescriptor>,
    time: Dataset,
    value: Dataset,
}

impl NexusSchematic for Log {
    const CLASS: &str = nexus_class::LOG;

    type Settings = (TypeDescriptor, RunLogChunkSize);

    fn build_group_structure(
        group: &Group,
        (type_descriptor, chunk_size): &Self::Settings,
    ) -> NexusHDF5Result<Self> {
        Ok(Self {
            type_descriptor: Some(type_descriptor.clone()),
            time: group.create_resizable_empty_dataset::<i64>("time", *chunk_size)?,
            value: group.create_dynamic_resizable_empty_dataset(
                "value",
                type_descriptor,
                *chunk_size,
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            type_descriptor: None,
            time: group.get_dataset("time")?,
            value: group.get_dataset("value")?,
        })
    }
}

impl NexusMessageHandler<LogWithOrigin<'_, f144_LogData<'_>>> for Log {
    fn handle_message(
        &mut self,
        message: &LogWithOrigin<'_, f144_LogData<'_>>,
    ) -> NexusHDF5Result<()> {
        message.append_timestamps(&self.time, message.get_origin())?;
        message.append_values(&self.value)?;
        Ok(())
    }
}

impl NexusMessageHandler<LogWithOrigin<'_, se00_SampleEnvironmentData<'_>>> for Log {
    fn handle_message(
        &mut self,
        message: &LogWithOrigin<'_, se00_SampleEnvironmentData<'_>>,
    ) -> NexusHDF5Result<()> {
        message.append_timestamps(&self.time, message.get_origin())?;
        message.append_values(&self.value)?;
        Ok(())
    }
}

impl NexusMessageHandler<LogWithOrigin<'_, SampleEnvironmentLog<'_>>> for Log {
    fn handle_message(
        &mut self,
        message: &LogWithOrigin<'_, SampleEnvironmentLog<'_>>,
    ) -> NexusHDF5Result<()> {
        match message.deref() {
            SampleEnvironmentLog::LogData(data) => {
                self.handle_message(&data.as_ref_with_origin(message.get_origin()))
            }
            SampleEnvironmentLog::SampleEnvironmentData(data) => {
                self.handle_message(&data.as_ref_with_origin(message.get_origin()))
            }
        }
    }
}

pub(crate) struct AlarmLog {
    alarm_severity: Dataset,
    alarm_status: Dataset,
    alarm_time: Dataset,
}

impl NexusSchematic for AlarmLog {
    const CLASS: &str = nexus_class::LOG;
    
    type Settings = AlarmChunkSize;
    
    fn build_group_structure(group: &Group, &alarm_chunk_size: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            alarm_severity: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_severity",
                alarm_chunk_size,
            )?,
            alarm_status: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_status",
                alarm_chunk_size,
            )?,
            alarm_time: group
                .create_resizable_empty_dataset::<i64>("alarm_time", alarm_chunk_size)?
        })
    }
    
    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            alarm_severity: group.get_dataset("alarm_severity")?,
            alarm_status: group.get_dataset("alarm_status")?,
            alarm_time: group.get_dataset("alarm_time")?
        })
    }
}

impl NexusMessageHandler<LogWithOrigin<'_, Alarm<'_>>> for AlarmLog {
    fn handle_message(
        &mut self,
        message: &LogWithOrigin<'_, Alarm<'_>>,
    ) -> NexusHDF5Result<()> {
        message.append_timestamp(&self.alarm_time, message.get_origin())?;
        message.append_severity(&self.alarm_severity)?;
        message.append_message(&self.alarm_status)?;
        Ok(())
    }
}

pub(crate) struct ValueLog {
    group: Group,
    alarm: Option<AlarmLog>,
    log: Option<Log>,
}

impl NexusSchematic for ValueLog {
    const CLASS: &str = nexus_class::LOG;

    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self{
            group: group.clone(),
            alarm: None,
            log: None,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
            alarm: AlarmLog::populate_group_structure(group).ok(),
            log: Log::populate_group_structure(group).ok(),
        })
    }
}

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for ValueLog {
    fn handle_message(
        &mut self,
        message: &PushSampleEnvironmentLog<'_>,
    ) -> NexusHDF5Result<()> {
        if self.log.is_none() {
            self.log = Some(Log::build_group_structure(&self.group, &(message.0.get_type_descriptor()?, message.1.selog))?);
        }
        match message.0.deref() {
            SampleEnvironmentLog::LogData(data) => {
                self.log.as_mut().expect("log exists, this shouldn't happen").handle_message(&data.as_ref_with_origin(message.0.get_origin()))
            }
            SampleEnvironmentLog::SampleEnvironmentData(data) => {
                self.log.as_mut().expect("log exists, this shouldn't happen").handle_message(&data.as_ref_with_origin(message.0.get_origin()))
            }
        }
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for ValueLog {
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        if self.alarm.is_none() {
            self.alarm = Some(AlarmLog::build_group_structure(&self.group, &message.1.alarm)?);
        }
        self.alarm.as_mut().expect("alarm exists, this shouldn't happen").handle_message(message.0)
    }
}
