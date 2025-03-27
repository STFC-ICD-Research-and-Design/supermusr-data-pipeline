use hdf5::{
    types::{TypeDescriptor, VarLenUnicode},
    Dataset, Group,
};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    hdf5_handlers::{ConvertResult, NexusHDF5Result},
    nexus::{nexus_class, LogMessage, NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::PushSampleEnvironmentLog, GroupExt, NexusDateTime, SampleEnvironmentLog},
    NexusSettings,
};

pub(crate) struct Log {
    time: Dataset,
    value: Dataset,
}

impl NexusSchematic for Log {
    const CLASS: &str = nexus_class::LOG;

    type Settings = (TypeDescriptor, usize);

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

impl NexusMessageHandler<(&f144_LogData<'_>, &NexusDateTime)> for Log {
    fn handle_message(&mut self, (message, origin_datetime): &(&f144_LogData<'_>, &NexusDateTime)) -> NexusHDF5Result<()> {
        message.append_timestamps(&self.time, 1, origin_datetime)?;
        message.append_values(&self.value)?;
        Ok(())
    }
}

impl NexusMessageHandler<se00_SampleEnvironmentData<'_>> for Log {
    fn handle_message(&mut self, message: &se00_SampleEnvironmentData) -> NexusHDF5Result<()> {
        message.append_timestamps(&self.time)?;
        message.append_values(&self.value)?;
        Ok(())
    }
}

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for Log {
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog) -> NexusHDF5Result<()> {
        let PushSampleEnvironmentLog(selog, time, settings) = message;

        match selog {
            SampleEnvironmentLog::LogData(data) => self.handle_message(data),
            SampleEnvironmentLog::SampleEnvironmentData(data) => self.handle_message(data),
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

    type Settings = (TypeDescriptor, usize, usize);

    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self> {
        let (type_descriptor, alarm_chunk_size, log_chunk_size) = settings;
        Ok(Self {
            alarm_severity: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_severity",
                *alarm_chunk_size,
            )?,
            alarm_status: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_status",
                *alarm_chunk_size,
            )?,
            alarm_time: group
                .create_resizable_empty_dataset::<i64>("alarm_time", *alarm_chunk_size)?,
            log: Log::build_group_structure(group, &(type_descriptor.clone(), *log_chunk_size))?,
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

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for ValueLog {
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog) -> NexusHDF5Result<()> {
        let PushSampleEnvironmentLog(selog, time, settings) = message;
        match selog {
            SampleEnvironmentLog::LogData(data) => self.log.handle_message(data),
            SampleEnvironmentLog::SampleEnvironmentData(data) => self.log.handle_message(data),
        }
    }
}

impl NexusMessageHandler<Alarm<'_>> for ValueLog {
    fn handle_message(&mut self, message: &Alarm) -> NexusHDF5Result<()> {
        todo!()
    }
}
