use std::path::Path;

use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use supermusr_streaming_types::ecs_6s4t_run_stop_generated::RunStop;

use super::{hdf5::RunFile, run_parameters::RunParameters, GenericEventMessage};

#[derive(Debug)]
pub(crate) struct Run {
    parameters: RunParameters,
}

impl Run {
    pub(crate) fn new(filename: &Path, parameters: RunParameters) -> Result<Self> {
        let mut hdf5 = RunFile::new(filename, &parameters.run_name)?;
        hdf5.init(&parameters)?;
        hdf5.close()?;

        Ok(Self { parameters })
    }

    pub(crate) fn push_message(
        &mut self,
        filename: &Path,
        message: &GenericEventMessage,
    ) -> Result<()> {
        let mut hdf5 = RunFile::open(filename, &self.parameters.run_name)?;
        hdf5.push_message(&self.parameters, message)?;
        hdf5.close()?;

        self.parameters.update_time_completed();
        Ok(())
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.parameters.run_name
    }

    pub(crate) fn has_run_stop(&self) -> bool {
        self.parameters.run_stop_parameters.is_some()
    }

    pub(crate) fn set_stop_if_valid(&mut self, filename: &Path, data: RunStop<'_>) -> Result<()> {
        self.parameters.set_stop_if_valid(data)?;
        let mut hdf5 = RunFile::open(filename, &self.parameters.run_name)?;
        hdf5.set_end_time(data.stop_time().try_into()?)?;
        hdf5.close()?;
        Ok(())
    }

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> Result<bool> {
        self.parameters.is_message_timestamp_valid(timestamp)
    }

    pub(crate) fn is_ready_to_write(&self, now: &DateTime<Utc>, delay: &Duration) -> bool {
        self.parameters
            .run_stop_parameters
            .as_ref()
            .map(|run_stop_parameters| *now - run_stop_parameters.time_completed > *delay)
            .unwrap_or(false)
    }

    pub(crate) fn close(self) -> Result<()> {
        //self.hdf5.close()
        Ok(())
    }
}
