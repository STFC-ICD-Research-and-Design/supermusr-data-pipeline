use crate::{
    hdf5_handlers::{GroupExt, NexusHDF5Result},
    nexus::{AlarmMessage, NexusClass, NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::PushAlarm, AlarmChunkSize},
};
use hdf5::{types::VarLenUnicode, Dataset, Group};

pub(crate) struct AlarmLog {
    alarm_severity: Dataset,
    alarm_status: Dataset,
    alarm_time: Dataset,
}

impl NexusSchematic for AlarmLog {
    const CLASS: NexusClass = NexusClass::Log;
    type Settings = AlarmChunkSize;

    fn build_group_structure(
        group: &Group,
        &alarm_chunk_size: &Self::Settings,
    ) -> NexusHDF5Result<Self> {
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
                .create_resizable_empty_dataset::<i64>("alarm_time", alarm_chunk_size)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            alarm_severity: group.get_dataset("alarm_severity")?,
            alarm_status: group.get_dataset("alarm_status")?,
            alarm_time: group.get_dataset("alarm_time")?,
        })
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for AlarmLog {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        message.append_timestamp_to(&self.alarm_time, message.origin)?;
        message.append_severity_to(&self.alarm_severity)?;
        message.append_message_to(&self.alarm_status)?;
        Ok(())
    }
}
