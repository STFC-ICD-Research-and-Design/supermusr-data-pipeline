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
    nexus::{nexus_class, LogMessage, LogWithOrigin, NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::ValueLogSettings, RunLogChunkSize, SampleEnvironmentLog},
};

pub(crate) struct Log {
    time: Dataset,
    value: Dataset,
}

impl NexusSchematic for Log {
    const CLASS: &str = nexus_class::LOG;

    type Settings = (TypeDescriptor, RunLogChunkSize);

    fn build_group_structure(
        group: &Group,
        (type_descriptior, chunk_size): &Self::Settings,
    ) -> NexusHDF5Result<Self> {
        Ok(Self {
            time: group.create_resizable_empty_dataset::<i64>("time", *chunk_size)?,
            value: group.create_dynamic_resizable_empty_dataset(
                "value",
                type_descriptior,
                *chunk_size,
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
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

pub(crate) struct ValueLog {
    alarm_severity: Dataset,
    alarm_status: Dataset,
    alarm_time: Dataset,
    log: Log,
}

impl NexusSchematic for ValueLog {
    const CLASS: &str = nexus_class::LOG;

    type Settings = ValueLogSettings;

    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self> {
        let (type_descriptor, runlog_chunk_sizes, alarm_chunk_sizes) = settings;
        Ok(Self {
            alarm_severity: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_severity",
                *alarm_chunk_sizes,
            )?,
            alarm_status: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_status",
                *alarm_chunk_sizes,
            )?,
            alarm_time: group
                .create_resizable_empty_dataset::<i64>("alarm_time", *alarm_chunk_sizes)?,
            log: Log::build_group_structure(
                group,
                &(type_descriptor.clone(), *runlog_chunk_sizes),
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            alarm_severity: group.get_dataset("alarm_severity")?,
            alarm_status: group.get_dataset("alarm_status")?,
            alarm_time: group.get_dataset("alarm_time")?,
            log: Log::populate_group_structure(group)?,
        })
    }
}

impl NexusMessageHandler<LogWithOrigin<'_, SampleEnvironmentLog<'_>>> for ValueLog {
    fn handle_message(
        &mut self,
        message: &LogWithOrigin<'_, SampleEnvironmentLog<'_>>,
    ) -> NexusHDF5Result<()> {
        match message.deref() {
            SampleEnvironmentLog::LogData(data) => {
                self.log.handle_message(&data.as_ref_with_origin(message.get_origin()))
            }
            SampleEnvironmentLog::SampleEnvironmentData(data) => {
                self.log.handle_message(&data.as_ref_with_origin(message.get_origin()))
            }
        }
    }
}

impl NexusMessageHandler<Alarm<'_>> for ValueLog {
    fn handle_message(&mut self, message: &Alarm) -> NexusHDF5Result<()> {
        todo!()
    }
}
