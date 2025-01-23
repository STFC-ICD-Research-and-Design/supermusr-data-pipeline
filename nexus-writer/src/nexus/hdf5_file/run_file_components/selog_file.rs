use super::timeseries_file::TimeSeriesDataSource;
use crate::nexus::{
    hdf5_file::{
        hdf5_writer::{DatasetExt, GroupExt},
        run_file_components::timeseries_file::get_dataset_builder,
    },
    nexus_class as NX, NexusSettings,
};
use hdf5::{types::VarLenUnicode, Group, SimpleExtents};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct SeLog {
    parent: Group,
}

impl SeLog {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new_selog(parent: &Group) -> anyhow::Result<Self> {
        let logs = parent.add_new_group_to("selog", NX::SELOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_selog(parent: &Group) -> anyhow::Result<Self> {
        let parent = parent.get_group("selog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_new_selogdata_in_selog(
        &mut self,
        selog: &se00_SampleEnvironmentData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<Group> {
        self.parent
            .add_new_group_to(selog.name(), NX::SELOG_BLOCK)
            .and_then(|parent_group| {
                let group = parent_group.add_new_group_to("value_log", NX::LOG)?;
                let time = group.create_resizable_dataset::<i32>(
                    "time",
                    0,
                    nexus_settings.seloglist_chunk_size,
                )?;
                selog.write_initial_timestamp(&time)?;
                get_dataset_builder(&selog.get_hdf5_type()?, &group)?
                    .shape(SimpleExtents::resizable(vec![0]))
                    .chunk(nexus_settings.seloglist_chunk_size)
                    .create("value")?;

                group.create_resizable_dataset::<VarLenUnicode>(
                    "alarm_severity",
                    0,
                    nexus_settings.alarmlist_chunk_size,
                )?;
                group.create_resizable_dataset::<VarLenUnicode>(
                    "alarm_status",
                    0,
                    nexus_settings.alarmlist_chunk_size,
                )?;
                group.create_resizable_dataset::<i64>(
                    "alarm_time",
                    0,
                    nexus_settings.alarmlist_chunk_size,
                )?;

                Ok::<_, anyhow::Error>(parent_group)
            })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_alarm_to_selog(&mut self, alarm: Alarm) -> anyhow::Result<()> {
        if let Some(source_name) = alarm.source_name() {
            if let Ok(timeseries) = self
                .parent
                .group(source_name)
                .and_then(|group| group.group("value_log"))
            {
                let alarm_time = timeseries.get_dataset("alarm_time")?;
                let alarm_status = timeseries.get_dataset("alarm_status")?;
                let alarm_severity = timeseries.get_dataset("alarm_severity")?;
                alarm_time.append_slice(&[alarm.timestamp()])?;

                if let Some(severity) = alarm.severity().variant_name() {
                    alarm_severity.append_slice(&[severity.parse::<VarLenUnicode>()?])?;
                }

                alarm_status.resize(alarm_status.size() + 1)?;
                if let Some(message) = alarm.message() {
                    alarm_status.append_slice(&[message.parse::<VarLenUnicode>()?])?;
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_selogdata_to_selog(
        &mut self,
        selog: &se00_SampleEnvironmentData,
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        debug!("Type: {0:?}", selog.values_type());

        let timeseries = self
            .parent
            .get_group(selog.name())
            .or_else(|err| {
                self.create_new_selogdata_in_selog(selog, nexus_settings)
                    .map_err(|e| e.context(err))
            })?
            .get_group("value_log")?;
        let timestamps = timeseries.get_dataset("time")?;
        let values = timeseries.get_dataset("value")?;
        let num_values = selog.write_values_to_dataset(&values)?;
        selog.write_timestamps_to_dataset(&timestamps, num_values)?;
        Ok(())
    }
}
