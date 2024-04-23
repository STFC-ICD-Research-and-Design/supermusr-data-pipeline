use super::{hdf5_file::RunFile, NexusSettings, RunParameters};
use crate::event_message::GenericEventMessage;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::{fmt::Debug, path::Path};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};

const TRACING_CLASS: &str = "NexusWriter::Run";

pub(crate) trait RunLike: Debug + AsRef<Run> + AsMut<Run> {
    fn new(
        filename: Option<&Path>,
        parameters: RunParameters,
        settings: &NexusSettings,
    ) -> Result<Self>
    where
        Self: Sized;
}

#[derive(Debug)]
pub(crate) struct Run {
    parameters: RunParameters,
}

impl AsRef<Self> for Run {
    fn as_ref(&self) -> &Run {
        self
    }
}
impl AsMut<Self> for Run {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl RunLike for Run {
    #[tracing::instrument(fields(class = TRACING_CLASS))]
    fn new(
        filename: Option<&Path>,
        parameters: RunParameters,
        settings: &NexusSettings,
    ) -> Result<Self> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::new(filename, &parameters.run_name, settings)?;
            hdf5.init(&parameters)?;
            hdf5.close()?;
        }
        Ok(Self { parameters })
    }
}

impl Run {
    #[cfg(test)]
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn push_logdata(
        &mut self,
        filename: Option<&Path>,
        logdata: &f144_LogData,
        settings: &NexusSettings,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open(filename, &self.parameters.run_name)?;
            hdf5.push_logdata(settings, logdata)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn push_selogdata(
        &mut self,
        filename: Option<&Path>,
        logdata: se00_SampleEnvironmentData,
        settings: &NexusSettings,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open(filename, &self.parameters.run_name)?;
            hdf5.push_selogdata(settings, logdata)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn push_message(
        &mut self,
        filename: Option<&Path>,
        message: &GenericEventMessage,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open(filename, &self.parameters.run_name)?;
            hdf5.push_message(&self.parameters, message)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn get_name(&self) -> &str {
        &self.parameters.run_name
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn has_run_stop(&self) -> bool {
        self.parameters.run_stop_parameters.is_some()
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn set_stop_if_valid(
        &mut self,
        filename: Option<&Path>,
        data: RunStop<'_>,
    ) -> Result<()> {
        self.parameters.set_stop_if_valid(data)?;
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open(filename, &self.parameters.run_name)?;

            hdf5.set_end_time(
                &self
                    .parameters
                    .run_stop_parameters
                    .as_ref()
                    .unwrap()
                    .collect_until,
            )?;
            hdf5.close()?;
        }
        Ok(())
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        self.parameters.is_message_timestamp_valid(timestamp)
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn has_completed(&self, delay: &Duration) -> bool {
        self.parameters
            .run_stop_parameters
            .as_ref()
            .map(|run_stop_parameters| Utc::now() - run_stop_parameters.last_modified > *delay)
            .unwrap_or(false)
    }
}
