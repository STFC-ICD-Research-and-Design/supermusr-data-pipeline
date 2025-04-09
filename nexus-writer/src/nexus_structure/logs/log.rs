use std::ops::Deref;

use hdf5::{types::TypeDescriptor, Dataset, Group};
use supermusr_common::DigitizerId;

use crate::{
    error::FlatBufferMissingError,
    hdf5_handlers::{DatasetExt, GroupExt, NexusHDF5Error, NexusHDF5Result},
    nexus::{LogMessage, NexusClass, NexusMessageHandler, NexusSchematic},
    run_engine::{
        run_messages::{
            InternallyGeneratedLog, PushInternallyGeneratedLogWarning, PushRunLog,
            PushSampleEnvironmentLog,
        },
        NexusDateTime, SampleEnvironmentLog,
    },
};

pub(crate) struct LogSettings {
    pub(crate) type_descriptor: TypeDescriptor,
    pub(crate) chunk_size: usize,
}

pub(crate) struct Log {
    time: Dataset,
    value: Dataset,
}

impl NexusSchematic for Log {
    const CLASS: NexusClass = NexusClass::Log;

    type Settings = LogSettings;

    fn build_group_structure(
        group: &Group,
        LogSettings {
            type_descriptor,
            chunk_size,
        }: &Self::Settings,
    ) -> NexusHDF5Result<Self> {
        Ok(Self {
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
            time: group.get_dataset("time")?,
            value: group.get_dataset("value")?,
        })
    }
}

impl NexusMessageHandler<PushRunLog<'_>> for Log {
    fn handle_message(&mut self, message: &PushRunLog<'_>) -> NexusHDF5Result<()> {
        message.append_timestamps_to(&self.time, message.origin)?;
        message.append_values_to(&self.value)?;
        Ok(())
    }
}

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for Log {
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog<'_>) -> NexusHDF5Result<()> {
        match message.deref() {
            SampleEnvironmentLog::LogData(f144_message) => {
                f144_message.append_timestamps_to(&self.time, message.origin)?;
                f144_message.append_values_to(&self.value)?;
            }
            SampleEnvironmentLog::SampleEnvironmentData(se00_message) => {
                se00_message.append_timestamps_to(&self.time, message.origin)?;
                se00_message.append_values_to(&self.value)?;
            }
        }
        Ok(())
    }
}

impl NexusMessageHandler<PushInternallyGeneratedLogWarning<'_>> for Log {
    fn handle_message(
        &mut self,
        message: &PushInternallyGeneratedLogWarning<'_>,
    ) -> NexusHDF5Result<()> {
        match message.message {
            InternallyGeneratedLog::RunResume { resume_time } => {
                self.time.append_value(
                    (*resume_time - message.origin)
                        .num_nanoseconds()
                        .unwrap_or_default(),
                )?;
                self.value.append_value(0)?; // This is a default value, I'm not sure if this field is needed
            }
            InternallyGeneratedLog::IncompleteFrame { frame } => {
                let timestamp: NexusDateTime = (*frame.metadata().timestamp().ok_or(FlatBufferMissingError::Timestamp)?)
                .try_into()?;

                // Recalculate time_zero of the frame to be relative to the offset value
                // (set at the start of the run).
                let time_zero =
                    (timestamp - message.origin)
                        .num_nanoseconds()
                        .ok_or_else(|| {
                            NexusHDF5Error::flatbuffer_timestamp_convert_to_nanoseconds(
                                timestamp - message.origin,
                            )
                        })?;

                let digitisers_present = frame
                    .digitizers_present()
                    .unwrap_or_default()
                    .iter()
                    .map(|x| DigitizerId::to_string(&x))
                    .collect::<Vec<_>>()
                    .join(",")
                    .parse::<hdf5::types::VarLenUnicode>()?;

                self.time.append_value(time_zero)?;
                self.value.append_value(digitisers_present)?;
            }
            InternallyGeneratedLog::AbortRun { stop_time_ms } => {
                let time = (message
                    .origin
                    .timestamp_nanos_opt()
                    .map(|origin_time_ns| 1_000_000 * stop_time_ms - origin_time_ns)
                    .unwrap_or_default() as f64)
                    / 1_000_000_000.0;
                self.time.append_value(time)?;
                self.value.append_value(0)?; // This is a default value, I'm not sure if this field is needed
            }
        }
        Ok(())
    }
}
