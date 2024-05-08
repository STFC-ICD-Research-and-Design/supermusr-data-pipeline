use super::{add_new_group_to, timeseries_file::TimeSeriesDataSource};
use crate::nexus::{
    hdf5_file::{
        hdf5_writer::create_resizable_dataset,
        run_file_components::timeseries_file::get_dataset_builder,
    },
    nexus_class as NX,
};
use anyhow::Result;
use hdf5::{Group, SimpleExtents};
use std::fmt::Debug;
use supermusr_streaming_types::ecs_se00_data_generated::se00_SampleEnvironmentData;
use tracing::{debug, warn};

#[derive(Debug)]
pub(crate) struct SeLog {
    parent: Group,
}

impl SeLog {
    #[tracing::instrument]
    pub(crate) fn new_selog(parent: &Group) -> Result<Self> {
        let logs = add_new_group_to(parent, "selog", NX::SELOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument]
    pub(crate) fn open_selog(parent: &Group) -> Result<Self> {
        let parent = parent.group("selog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip(self))]
    fn create_new_selogdata_in_selog(
        &mut self,
        selog: &se00_SampleEnvironmentData,
    ) -> Result<Group> {
        add_new_group_to(&self.parent, selog.name(), NX::SELOG_BLOCK).and_then(|parent_group| {
            let group = add_new_group_to(&parent_group, "value_log", NX::RUNLOG)?;
            let time = create_resizable_dataset::<i32>(&group, "time", 0, 1024)?;
            selog.write_initial_timestamp(&time)?;
            get_dataset_builder(&selog.get_hdf5_type()?, &group)?
                .shape(SimpleExtents::resizable(vec![0]))
                .chunk(1024)
                .create("value")?;
            Ok::<_, anyhow::Error>(parent_group)
        })
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn push_selogdata_to_selog(
        &mut self,
        selog: &se00_SampleEnvironmentData,
    ) -> Result<()> {
        debug!("Type: {0:?}", selog.values_type());

        let timeseries = self
            .parent
            .group(selog.name())
            .or_else(|err| {
                self.create_new_selogdata_in_selog(selog)
                    .map_err(|e| e.context(err))
            })?
            .group("value_log")?;
        let timestamps = timeseries.dataset("time")?;
        let values = timeseries.dataset("value")?;
        let num_values = selog.write_values_to_dataset(&values)?;
        selog.write_timestamps_to_dataset(&timestamps, num_values)?;
        Ok(())
    }
}
