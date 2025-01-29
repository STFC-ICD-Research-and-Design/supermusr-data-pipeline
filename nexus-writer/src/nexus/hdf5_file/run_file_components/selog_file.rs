use super::timeseries_file::{adjust_nanoseconds_by_origin_to_sec, TimeSeriesDataSource};
use crate::nexus::{
    hdf5_file::{
        error::{ConvertResult, NexusHDF5Result},
        hdf5_writer::{DatasetExt, GroupExt, HasAttributesExt},
    },
    nexus_class as NX, NexusDateTime, NexusSettings,
};
use hdf5::{types::VarLenUnicode, Group};
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
    pub(crate) fn new_selog(parent: &Group) -> NexusHDF5Result<Self> {
        let logs = parent.add_new_group_to("selog", NX::SELOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_selog(parent: &Group) -> NexusHDF5Result<Self> {
        let parent = parent.get_group("selog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_alarm_to_selog(
        &mut self,
        alarm: Alarm,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        if let Some(source_name) = alarm.source_name() {
            let seblock = self
                .parent
                .get_group_or_create_new(source_name, NX::SELOG_BLOCK)?;
            let value_log = seblock.get_group_or_create_new("value_log", NX::LOG)?;

            let alarm_time = value_log.get_dataset_or_else("alarm_time", |group| {
                let alarm_time = group.create_resizable_empty_dataset::<f32>(
                    "alarm_time",
                    nexus_settings.alarmlist_chunk_size,
                )?;
                let start = origin_time.to_rfc3339();
                alarm_time.add_attribute_to("Start", &start)?;
                alarm_time.add_attribute_to("Units", "second")?;
                Ok(alarm_time)
            })?;

            let alarm_status = value_log.get_dataset_or_else("alarm_status", |group| {
                group.create_resizable_empty_dataset::<VarLenUnicode>(
                    "alarm_status",
                    nexus_settings.alarmlist_chunk_size,
                )
            })?;

            let alarm_severity = value_log.get_dataset_or_else("alarm_severity", |group| {
                group.create_resizable_empty_dataset::<VarLenUnicode>(
                    "alarm_severity",
                    nexus_settings.alarmlist_chunk_size,
                )
            })?;

            alarm_time.append_slice(&[adjust_nanoseconds_by_origin_to_sec(alarm.timestamp(), origin_time)])?;

            if let Some(severity) = alarm.severity().variant_name() {
                alarm_severity.append_slice(&[severity
                    .parse::<VarLenUnicode>()
                    .err_group(&self.parent)?])?;
            } else {
                alarm_severity.append_slice(&[VarLenUnicode::default()])?;
            }

            if let Some(message) = alarm.message() {
                alarm_status
                    .append_slice(&[message.parse::<VarLenUnicode>().err_group(&self.parent)?])?;
            } else {
                alarm_severity.append_slice(&[VarLenUnicode::default()])?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_selogdata_to_selog(
        &mut self,
        selog: &se00_SampleEnvironmentData,
        origin_time: &NexusDateTime,
        nexus_settings: &NexusSettings,
    ) -> NexusHDF5Result<()> {
        debug!("Type: {0:?}", selog.values_type());

        let seblock = self
            .parent
            .get_group_or_create_new(selog.name(), NX::SELOG_BLOCK)?;
        let value_log = seblock.get_group_or_create_new("value_log", NX::LOG)?;

        let timestamps = value_log.get_dataset_or_else("time", |_| {
            let times = value_log.create_resizable_empty_dataset::<f32>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            let start = origin_time.to_rfc3339();
            times.add_attribute_to("Start", &start)?;
            times.add_attribute_to("Units", "second")?;
            Ok(times)
        })?;

        let values = value_log.get_dataset_or_create_dynamic_resizable_empty_dataset(
            "value",
            &selog.get_hdf5_type().err_group(&self.parent)?,
            nexus_settings.seloglist_chunk_size,
        )?;

        let num_values = selog.write_values_to_dataset(&values)?;
        selog.write_timestamps_to_dataset(&timestamps, num_values, origin_time)?;
        Ok(())
    }
}
