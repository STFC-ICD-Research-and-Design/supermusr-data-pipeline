use super::{hdf5_file::RunFile, RunParameters};
use crate::event_message::GenericEventMessage;
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use std::path::Path;
use supermusr_common::spanned::{SpanOnce, Spanned, SpannedMut};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
}

impl Run {
    #[tracing::instrument]
    pub(crate) fn new_run(filename: Option<&Path>, parameters: RunParameters) -> Result<Self> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::new_runfile(filename, &parameters.run_name)?;
            hdf5.init(&parameters)?;
            hdf5.close()?;
        }
        Ok(Self {
            span: Default::default(),
            parameters,
        })
    }
    #[cfg(test)]
    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn push_logdata_to_run(
        &mut self,
        filename: Option<&Path>,
        logdata: &f144_LogData,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_logdata_to_runfile(logdata)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn push_alarm_to_run(
        &mut self,
        filename: Option<&Path>,
        alarm: Alarm,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_alarm_to_runfile(alarm)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    pub(crate) fn push_selogdata(
        &mut self,
        filename: Option<&Path>,
        logdata: se00_SampleEnvironmentData,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_selogdata(logdata)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    pub(crate) fn push_message(
        &mut self,
        filename: Option<&Path>,
        message: &GenericEventMessage,
    ) -> Result<()> {
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;
            hdf5.push_message_to_runfile(&self.parameters, message)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn get_name(&self) -> &str {
        &self.parameters.run_name
    }

    pub(crate) fn has_run_stop(&self) -> bool {
        self.parameters.run_stop_parameters.is_some()
    }

    pub(crate) fn set_stop_if_valid(
        &mut self,
        filename: Option<&Path>,
        data: RunStop<'_>,
    ) -> Result<()> {
        self.parameters.set_stop_if_valid(data)?;
        if let Some(filename) = filename {
            let mut hdf5 = RunFile::open_runfile(filename, &self.parameters.run_name)?;

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

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> bool {
        self.parameters.is_message_timestamp_valid(timestamp)
    }

    pub(crate) fn has_completed(&self, delay: &Duration) -> bool {
        self.parameters
            .run_stop_parameters
            .as_ref()
            .map(|run_stop_parameters| Utc::now() - run_stop_parameters.last_modified > *delay)
            .unwrap_or(false)
    }
}

impl Spanned for Run {
    fn span(&self) -> &SpanOnce {
        &self.span
    }
}

impl SpannedMut for Run {
    fn span_mut(&mut self) -> &mut SpanOnce {
        &mut self.span
    }
}
