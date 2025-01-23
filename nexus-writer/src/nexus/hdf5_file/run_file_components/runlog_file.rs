use super::timeseries_file::TimeSeriesDataSource;
use crate::nexus::{
    hdf5_file::{
        hdf5_writer::{DatasetExt, GroupExt},
        run_file_components::timeseries_file::get_dataset_builder,
    },
    nexus_class as NX, NexusSettings,
};
use hdf5::{
    types::{IntSize, TypeDescriptor},
    Group, SimpleExtents,
};
use supermusr_common::DigitizerId;
use supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData;
use tracing::debug;

#[derive(Debug)]
pub(crate) struct RunLog {
    parent: Group,
}

impl RunLog {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new_runlog(parent: &Group) -> anyhow::Result<Self> {
        let logs = parent.add_new_group_to("runlog", NX::RUNLOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_runlog(parent: &Group) -> anyhow::Result<Self> {
        let parent = parent.group("runlog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_logdata_to_runlog(
        &mut self,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        debug!("Type: {0:?}", logdata.value_type());

        let timeseries = self.parent.group(logdata.source_name()).or_else(|err| {
            debug!(
                "Cannot find {0}. Creating new group.",
                logdata.source_name()
            );

            let group = self
                .parent
                .add_new_group_to(logdata.source_name(), NX::LOG)
                .map_err(|e| e.context(err))?;

            let time = group.create_resizable_empty_dataset::<u64>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            logdata.write_initial_timestamp(&time)?;
            get_dataset_builder(&logdata.get_hdf5_type()?, &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(nexus_settings.runloglist_chunk_size)
                .create("value")?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.get_dataset("time")?;
        let values = timeseries.get_dataset("value")?;

        logdata.write_values_to_dataset(&values)?;
        logdata.write_timestamps_to_dataset(&timestamps, 1)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn set_aborted_run_warning(
        &mut self,
        stop_time: i32,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_RunAborted";
        let timeseries = self.parent.group(LOG_NAME).or_else(|err| {
            let group = self
                .parent
                .add_new_group_to(LOG_NAME, NX::LOG)
                .map_err(|e| e.context(err))?;

            let _time = group.create_resizable_empty_dataset::<u64>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            get_dataset_builder(&TypeDescriptor::Unsigned(IntSize::U1), &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(nexus_settings.runloglist_chunk_size)
                .create("value")?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.get_dataset("time")?;
        let values = timeseries.get_dataset("value")?;

        timestamps.set_slice_to(&[stop_time])?;
        values.set_slice_to(&[0])?; // This is a default value, I'm not sure if this field is needed

        Ok(())
    }

    pub(crate) fn push_incomplete_frame_log(
        &mut self,
        event_time_zero: u64,
        digitisers_present: Vec<DigitizerId>,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_DigitisersPresentInIncompleteFrame";
        let timeseries = self.parent.group(LOG_NAME).or_else(|err| {
            debug!("Cannot find {LOG_NAME}. Creating new group.");

            let group = self
                .parent
                .add_new_group_to(LOG_NAME, NX::LOG)
                .map_err(|e| e.context(err))?;

            group.create_resizable_empty_dataset::<u64>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            group.create_resizable_empty_dataset::<hdf5::types::VarLenUnicode>(
                "value",
                nexus_settings.runloglist_chunk_size,
            )?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.get_dataset("time")?;
        let values = timeseries.get_dataset("value")?;

        if timestamps.size() != values.size() {
            anyhow::bail!(
                "time length ({}) and value length ({}) differ",
                timestamps.size(),
                values.size()
            )
        }

        timestamps.append_slice(&[event_time_zero])?;

        let value = digitisers_present
            .iter()
            .map(DigitizerId::to_string)
            .collect::<Vec<_>>()
            .join(",")
            .parse::<hdf5::types::VarLenUnicode>()?;
        values.append_slice(&[value])?;

        Ok(())
    }
}
