use super::timeseries_file::TimeSeriesDataSource;
use crate::nexus::{
    hdf5_file::{
        error::{ConvertResult, NexusHDF5Result},
        hdf5_writer::{DatasetExt, GroupExt, HasAttributesExt}, run_file_components::timeseries_file::adjust_nanoseconds_by_origin,
    },
    nexus_class as NX, NexusDateTime, NexusSettings,
};
use hdf5::{
    types::{IntSize, TypeDescriptor},
    Dataset, Group,
};
use supermusr_common::DigitizerId;
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;

#[derive(Debug)]
pub(crate) struct RunLog {
    parent: Group,
}

impl RunLog {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new_runlog(parent: &Group) -> NexusHDF5Result<Self> {
        let logs = parent.add_new_group_to("runlog", NX::RUNLOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_runlog(parent: &Group) -> NexusHDF5Result<Self> {
        let parent = parent.get_group("runlog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn create_runlog_group(
        &mut self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<(Dataset, Dataset)> {
        let runlog = self.parent.get_group_or_create_new(name, NX::RUNLOG)?;
        let timestamps = runlog.get_dataset_or_else("time", |_| {
            let times = runlog.create_resizable_empty_dataset::<i64>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            let start = origin_time.to_rfc3339();
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
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        let (timestamps, values) = self.create_runlog_group(
            logdata.source_name(),
            &logdata.get_hdf5_type().err_group(&self.parent)?,
            origin_time,
            nexus_settings,
        )?;

        logdata.write_values_to_dataset(&values)?;
        logdata.write_timestamps_to_dataset(&timestamps, 1, origin_time)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_aborted_run_warning(
        &mut self,
        stop_time_ms: i64,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_RunAborted";
        let (timestamps, values) = self.create_runlog_group(
            LOG_NAME,
            &TypeDescriptor::Unsigned(IntSize::U1),
            origin_time,
            nexus_settings,
        )?;

        timestamps.set_slice_to(&[adjust_nanoseconds_by_origin(1_000_000*stop_time_ms, origin_time)])?;
        values.set_slice_to(&[0])?; // This is a default value, I'm not sure if this field is needed

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_run_resumed_warning(
        &mut self,
        current_time: &NexusDateTime,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_RunResumed";
        let (timestamps, values) = self.create_runlog_group(
            LOG_NAME,
            &TypeDescriptor::Unsigned(IntSize::U1),
            origin_time,
            nexus_settings,
        )?;

        timestamps.set_slice_to(&[(*current_time - origin_time)
            .num_nanoseconds()
            .unwrap_or_default()])?;
        values.set_slice_to(&[0])?; // This is a default value, I'm not sure if this field is needed

        Ok(())
    }

    pub(crate) fn push_incomplete_frame_log(
        &mut self,
        event_time_zero: i64,
        digitisers_present: Vec<DigitizerId>,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_DigitisersPresentInIncompleteFrame";
        let (timestamps, values) = self.create_runlog_group(
            LOG_NAME,
            &TypeDescriptor::VarLenUnicode,
            origin_time,
            nexus_settings,
        )?;

        timestamps.set_slice_to(&[adjust_nanoseconds_by_origin(event_time_zero, origin_time)])?;

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
