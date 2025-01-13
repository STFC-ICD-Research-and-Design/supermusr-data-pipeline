use super::{add_new_group_to, timeseries_file::TimeSeriesDataSource};
use crate::nexus::{
    hdf5_file::{
        hdf5_writer::{create_resizable_dataset, set_slice_to},
        run_file_components::timeseries_file::get_dataset_builder,
    },
    nexus_class as NX, NexusSettings,
};
use hdf5::{
    types::{IntSize, TypeDescriptor},
    Group, H5Type, SimpleExtents,
};
use ndarray::s;
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
        let logs = add_new_group_to(parent, "runlog", NX::RUNLOG)?;
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

            let group = add_new_group_to(&self.parent, logdata.source_name(), NX::LOG)
                .map_err(|e| e.context(err))?;

            let time = create_resizable_dataset::<i32>(
                &group,
                "time",
                0,
                nexus_settings.runloglist_chunk_size,
            )?;
            logdata.write_initial_timestamp(&time)?;
            get_dataset_builder(&logdata.get_hdf5_type()?, &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(nexus_settings.runloglist_chunk_size)
                .create("value")?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.dataset("time")?;
        let values = timeseries.dataset("value")?;

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
            let group =
                add_new_group_to(&self.parent, LOG_NAME, NX::LOG).map_err(|e| e.context(err))?;

            let _time = create_resizable_dataset::<i32>(
                &group,
                "time",
                0,
                nexus_settings.runloglist_chunk_size,
            )?;
            get_dataset_builder(&TypeDescriptor::Unsigned(IntSize::U1), &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(nexus_settings.runloglist_chunk_size)
                .create("value")?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.dataset("time")?;
        let values = timeseries.dataset("value")?;

        set_slice_to(&timestamps, &[stop_time])?;
        set_slice_to(&values, &[0])?; // This is a default value, I'm not sure if this field is needed

        Ok(())
    }

    pub(crate) fn push_incomplete_frame_log(
        &mut self,
        event_time_zero: u64,
        digitisers_present: Vec<DigitizerId>,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        const LOGNAME: &str = "incomplete_frame_digitisers_present";
        let timeseries = self.parent.group(LOGNAME).or_else(|err| {
            debug!("Cannot find {LOGNAME}. Creating new group.");

            let group =
                add_new_group_to(&self.parent, LOGNAME, NX::LOG).map_err(|e| e.context(err))?;

            let _time = create_resizable_dataset::<i32>(
                &group,
                "time",
                0,
                nexus_settings.runloglist_chunk_size,
            )?;
            get_dataset_builder(&hdf5::types::VarLenUnicode::type_descriptor(), &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(nexus_settings.runloglist_chunk_size)
                .create("value")?;
            Ok::<_, anyhow::Error>(group)
        })?;
        let timestamps = timeseries.dataset("time")?;
        let values = timeseries.dataset("value")?;

        let next_message_slice = s![timestamps.size()..(timestamps.size() + 1)];

        timestamps.resize(timestamps.size() + 1)?;
        timestamps.write_slice(&[event_time_zero], next_message_slice)?;

        values.resize(values.size() + 1)?;
        values.write_slice(
            &[digitisers_present
                .iter()
                .map(DigitizerId::to_string)
                .collect::<Vec<_>>()
                .join(",")
                .parse::<hdf5::types::VarLenUnicode>()?],
            next_message_slice,
        )?;

        Ok(())
    }
}
