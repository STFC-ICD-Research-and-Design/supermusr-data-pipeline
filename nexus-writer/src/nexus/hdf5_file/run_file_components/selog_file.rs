use super::timeseries_file::TimeSeriesDataSource;
use crate::nexus::{
    hdf5_file::hdf5_writer::{DatasetExt, GroupExt, HasAttributesExt},
    nexus_class as NX, NexusSettings,
};
use chrono::{DateTime, Utc};
use hdf5::{types::VarLenUnicode, Group};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm, ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct SeLog {
    parent: Group,
    start_time: Option<DateTime<Utc>>,
}

impl SeLog {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn new_selog(parent: &Group) -> anyhow::Result<Self> {
        let logs = parent.add_new_group_to("selog", NX::SELOG)?;
        Ok(Self { parent: logs, start_time: None })
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn open_selog(parent: &Group) -> anyhow::Result<Self> {
        let parent = parent.get_group("selog")?;
        let start_time = parent.get_dataset("start_time")?;
        Ok(Self { parent, start_time: Some(start_time.get_datetime_from()?) })
    }

    #[tracing::instrument(skip_all, level = "trace")]
    pub(crate) fn init(&mut self, start_time: &DateTime<Utc>) {
        self.start_time = Some(start_time.clone());
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    pub(crate) fn push_alarm_to_selog(
        &mut self,
        alarm: Alarm,
        /* origin_time : DateTime<Utc> To Replace self.start_time */
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        if let Some(source_name) = alarm.source_name() {
            let seblock = self
                .parent
                .get_group_or_create_new(source_name, NX::SELOG_BLOCK)?;
            let value_log = seblock.get_group_or_create_new("value_log", NX::LOG)?;

            let alarm_time = value_log.get_dataset_or_else("alarm_time", |group| {
                group.create_resizable_empty_dataset::<i64>(
                    "alarm_time",
                    nexus_settings.alarmlist_chunk_size,
                )
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

            alarm_time.append_slice(&[alarm.timestamp()])?;

            if let Some(severity) = alarm.severity().variant_name() {
                alarm_severity.append_slice(&[severity.parse::<VarLenUnicode>()?])?;
            } else {
                alarm_severity.append_slice(&[VarLenUnicode::default()])?;
            }

            alarm_status.resize(alarm_status.size() + 1)?;
            if let Some(message) = alarm.message() {
                alarm_status.append_slice(&[message.parse::<VarLenUnicode>()?])?;
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
        /* origin_time : DateTime<Utc> To Replace self.start_time */
        nexus_settings: &NexusSettings,
    ) -> anyhow::Result<()> {
        debug!("Type: {0:?}", selog.values_type());

        let seblock = self
            .parent
            .get_group_or_create_new(selog.name(), NX::SELOG_BLOCK)?;
        let value_log = seblock.get_group_or_create_new("value_log", NX::LOG)?;

        let timestamps = value_log.get_dataset_or_else("time", |_| {
            let times = value_log.create_resizable_empty_dataset::<u64>(
                "time",
                nexus_settings.runloglist_chunk_size,
            )?;
            let start = self.start_time.unwrap_or_default().to_rfc3339();
            times.add_attribute_to("Start", &start)?;
            times.add_attribute_to("Units", "second")?;
            Ok(times)
        })?;

        let values = value_log.get_dataset_or_create_dynamic_resizable_empty_dataset(
            "value",
            &selog.get_hdf5_type()?,
            nexus_settings.seloglist_chunk_size,
        )?;

        let num_values = selog.write_values_to_dataset(&values)?;
        selog.write_timestamps_to_dataset(&timestamps, num_values)?;
        Ok(())
    }
}
