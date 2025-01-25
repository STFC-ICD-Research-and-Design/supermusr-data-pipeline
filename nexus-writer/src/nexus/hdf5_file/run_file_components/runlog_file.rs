use super::timeseries_file::TimeSeriesDataSource;
use crate::nexus::{
    hdf5_file::{
        error::{ConvertResult, NexusHDF5ErrorType, NexusHDF5Result},
        hdf5_writer::{DatasetExt, GroupExt, HasAttributesExt},
    },
    nexus_class as NX, NexusSettings,
};
use chrono::{DateTime, Utc};
use hdf5::{
    types::{IntSize, TypeDescriptor},
    Dataset, Group,
};
use supermusr_common::DigitizerId;
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

#[derive(Debug)]
pub(crate) struct RunLog {
    parent: Group,
    start_time: Option<DateTime<Utc>>,
}

impl RunLog {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new_runlog(parent: &Group) -> NexusHDF5Result<Self> {
        let logs = parent.add_new_group_to("runlog", NX::RUNLOG)?;
        Ok(Self {
            parent: logs,
            start_time: None,
        })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_runlog(parent: &Group) -> NexusHDF5Result<Self> {
        let parent = parent.get_group("runlog")?;
        let start_time = parent.get_dataset("start_time")?;
        Ok(Self {
            parent,
            start_time: Some(start_time.get_datetime_from()?),
        })
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn init(&mut self, start_time: &DateTime<Utc>) {
        self.start_time = Some(start_time.clone());
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn create_runlog_group(
        &mut self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<(Dataset, Dataset)> {
        let runlog = self.parent.get_group_or_create_new(name, NX::RUNLOG)?;
        let timestamps = runlog.get_dataset_or_else("time", |_| {
            let times = runlog.create_resizable_empty_dataset::<u64>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            let start = self.start_time.unwrap_or_default().to_rfc3339();
            times.add_attribute_to("Start", &start)?;
            times.add_attribute_to("Units", "second")?;
            Ok(times)
        })?;

        let values = runlog.get_dataset_or_create_dynamic_resizable_empty_dataset(
            "value",
            type_descriptor,
            nexus_settings.runloglist_chunk_size,
        )?;

        Ok((timestamps, values))
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_logdata_to_runlog(
        &mut self,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        let (timestamps, values) = self.create_runlog_group(
            logdata.source_name(),
            &logdata.get_hdf5_type().err_group(&self.parent)?,
            nexus_settings,
        )?;
        logdata.write_values_to_dataset(&values)?;
        logdata.write_timestamps_to_dataset(&timestamps, 1)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_aborted_run_warning(
        &mut self,
        stop_time: u64,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_RunAborted";
        let (timestamps, values) = self.create_runlog_group(
            LOG_NAME,
            &TypeDescriptor::Unsigned(IntSize::U1),
            nexus_settings,
        )?;
        timestamps.set_slice_to(&[stop_time])?;
        values.set_slice_to(&[0])?; // This is a default value, I'm not sure if this field is needed

        Ok(())
    }

    pub(crate) fn push_incomplete_frame_log(
        &mut self,
        event_time_zero: u64,
        digitisers_present: Vec<DigitizerId>,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_DigitisersPresentInIncompleteFrame";
        let (timestamps, values) =
            self.create_runlog_group(LOG_NAME, &TypeDescriptor::VarLenUnicode, nexus_settings)?;

        if timestamps.size() != values.size() {
            return Err(
                NexusHDF5ErrorType::FlatBufferInconsistentRunLogTimeValueSizes(
                    timestamps.size(),
                    values.size(),
                ),
            )
            .err_group(&self.parent)?;
        }

        timestamps.append_slice(&[event_time_zero])?;

        let value = digitisers_present
            .iter()
            .map(DigitizerId::to_string)
            .collect::<Vec<_>>()
            .join(",")
            .parse::<hdf5::types::VarLenUnicode>()
            .err_group(&self.parent)?;
        values.append_slice(&[value])?;

        Ok(())
    }
}
