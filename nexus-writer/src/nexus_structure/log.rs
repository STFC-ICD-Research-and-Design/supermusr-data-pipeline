use hdf5::{
    types::{TypeDescriptor, VarLenUnicode},
    Dataset, Group,
};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{nexus_class, NexusMessageHandler, NexusSchematic},
    run_engine::{GroupExt, SampleEnvironmentLog},
    NexusSettings,
};

pub(crate) struct Log {
    time: Dataset,
    value: Dataset,
}

impl NexusSchematic for Log {
    const CLASS: &str = nexus_class::LOG;

    type Settings = (TypeDescriptor, NexusSettings);

    fn build_group_structure(
        group: &Group,
        (type_descriptior, settings): &Self::Settings,
    ) -> NexusHDF5Result<Self> {
        Ok(Self {
            time: group.create_resizable_empty_dataset::<i64>(
                "time",
                settings.get_chunk_sizes().runloglist,
            )?,
            value: group.create_dynamic_resizable_empty_dataset(
                "value",
                type_descriptior,
                settings.get_chunk_sizes().runloglist,
            )?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            time: group.get_dataset("time")?,
            value: group.get_dataset("value")?,
        })
    }

    fn close_group() -> NexusHDF5Result<()> {
        Ok(())
    }
}

impl NexusMessageHandler<f144_LogData<'_>> for Log {
    fn handle_message(&mut self, message: &f144_LogData) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<se00_SampleEnvironmentData<'_>> for Log {
    fn handle_message(&mut self, message: &se00_SampleEnvironmentData) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<SampleEnvironmentLog<'_>> for Log {
    fn handle_message(&mut self, message: &SampleEnvironmentLog) -> NexusHDF5Result<()> {
        match message {
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

    type Settings = (TypeDescriptor, NexusSettings);

    fn build_group_structure(
        group: &Group,
        pair_settings: &Self::Settings,
    ) -> NexusHDF5Result<Self> {
        let (_, settings) = pair_settings;
        Ok(Self {
            alarm_severity: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_severity",
                settings.get_chunk_sizes().alarmlist,
            )?,
            alarm_status: group.create_resizable_empty_dataset::<VarLenUnicode>(
                "alarm_status",
                settings.get_chunk_sizes().alarmlist,
            )?,
            alarm_time: group.create_resizable_empty_dataset::<i64>(
                "alarm_time",
                settings.get_chunk_sizes().alarmlist,
            )?,
            log: Log::build_group_structure(group, pair_settings)?,
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

    fn close_group() -> NexusHDF5Result<()> {
        Ok(())
    }
}

impl NexusMessageHandler<SampleEnvironmentLog<'_>> for ValueLog {
    fn handle_message(&mut self, message: &SampleEnvironmentLog) -> NexusHDF5Result<()> {
        match message {
            SampleEnvironmentLog::LogData(f144_log_data) => self.log.handle_message(f144_log_data),
            SampleEnvironmentLog::SampleEnvironmentData(se00_sample_environment_data) => {
                self.log.handle_message(se00_sample_environment_data)
            }
        }
    }
}

impl NexusMessageHandler<Alarm<'_>> for ValueLog {
    fn handle_message(&mut self, message: &Alarm) -> NexusHDF5Result<()> {
        todo!()
    }
}
