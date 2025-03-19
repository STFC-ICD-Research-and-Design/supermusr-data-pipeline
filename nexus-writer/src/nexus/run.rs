use crate::message_handlers::SampleEnvironmentLog;

use super::{
    error::NexusWriterResult, hdf5_file::RunFile, NexusConfiguration, NexusDateTime, NexusSettings,
    RunParameters,
};
use chrono::{Duration, Utc};
use std::{io, path::Path};
use supermusr_common::spanned::{SpanOnce, SpanOnceError, Spanned, SpannedAggregator, SpannedMut};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData,
};
use tracing::{error, info, info_span, Span};

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
}

impl Run {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn new_run(
        nexus_settings: Option<&NexusSettings>,
        parameters: RunParameters,
        nexus_configuration: &NexusConfiguration,
    ) -> NexusWriterResult<Self> {
        if let Some(nexus_settings) = nexus_settings {
            let mut hdf5 = RunFile::new_runfile(
                nexus_settings.get_local_path(),
                &parameters.run_name,
                nexus_settings,
            )?;
            hdf5.init(&parameters, nexus_configuration)?;
            hdf5.close()?;
        }

        Ok(Self {
            span: Default::default(),
            parameters,
        })
    }

    pub(crate) fn resume_partial_run(
        nexus_settings: &NexusSettings,
        filename: &str,
    ) -> NexusWriterResult<Self> {
        let mut run = RunFile::open_runfile(nexus_settings.get_local_path(), filename)?;
        let parameters = run.extract_run_parameters()?;
        run.push_run_resumed_warning(&Utc::now(), &parameters.collect_from, nexus_settings)?;
        run.close()?;

        Ok(Self {
            span: Default::default(),
            parameters,
        })
    }

    pub(crate) fn parameters(&self) -> &RunParameters {
        &self.parameters
    }

    /// This method renames the path of LOCAL_PATH/temp/FILENAME.nxs to LOCAL_PATH/completed/FILENAME.nxs
    /// As these paths are on the same mount, no actual file move occurs,
    /// So this does not need to be async.
    pub(crate) fn move_to_completed(
        &self,
        temp_path: &Path,
        completed_path: &Path,
    ) -> io::Result<()> {
        let from_path = RunParameters::get_hdf5_filename(temp_path, &self.parameters.run_name);
        let to_path = RunParameters::get_hdf5_filename(completed_path, &self.parameters.run_name);

        info_span!(
            "Move To Completed",
            from_path = from_path.to_string_lossy().to_string(),
            to_path = to_path.to_string_lossy().to_string()
        )
        .in_scope(|| match std::fs::rename(from_path, to_path) {
            Ok(()) => {
                info!("File Move Succesful.");
                Ok(())
            }
            Err(e) => {
                error!("File Move Error {e}");
                Err(e)
            }
        })
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_logdata_to_run(
        &mut self,
        nexus_settings: Option<&NexusSettings>,
        logdata: &f144_LogData,
    ) -> NexusWriterResult<()> {
        if let Some(nexus_settings) = nexus_settings {
            let mut hdf5 =
                RunFile::open_runfile(nexus_settings.get_local_path(), &self.parameters.run_name)?;
            hdf5.push_logdata_to_runfile(logdata, &self.parameters.collect_from, nexus_settings)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_alarm_to_run(
        &mut self,
        nexus_settings: Option<&NexusSettings>,
        alarm: Alarm,
    ) -> NexusWriterResult<()> {
        if let Some(nexus_settings) = nexus_settings {
            let mut hdf5 =
                RunFile::open_runfile(nexus_settings.get_local_path(), &self.parameters.run_name)?;
            hdf5.push_alarm_to_runfile(alarm, &self.parameters.collect_from, nexus_settings)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        nexus_settings: Option<&NexusSettings>,
        logdata: SampleEnvironmentLog,
    ) -> NexusWriterResult<()> {
        if let Some(nexus_settings) = nexus_settings {
            let mut hdf5 =
                RunFile::open_runfile(nexus_settings.get_local_path(), &self.parameters.run_name)?;
            hdf5.push_selogdata(logdata, &self.parameters.collect_from, nexus_settings)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_frame_eventlist_message(
        &mut self,
        nexus_settings: Option<&NexusSettings>,
        message: &FrameAssembledEventListMessage,
    ) -> NexusWriterResult<()> {
        if let Some(nexus_settings) = nexus_settings {
            let mut hdf5 =
                RunFile::open_runfile(nexus_settings.get_local_path(), &self.parameters.run_name)?;
            hdf5.push_frame_eventlist_message_to_runfile(message)?;

            if !message.complete() {
                hdf5.push_incomplete_frame_warning(message, nexus_settings)?;
            }

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
        local_path: Option<&Path>,
        data: RunStop<'_>,
    ) -> NexusWriterResult<()> {
        self.parameters.set_stop_if_valid(data)?;

        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::open_runfile(local_path, &self.parameters.run_name)?;

            hdf5.set_end_time(
                &self
                    .parameters
                    .run_stop_parameters
                    .as_ref()
                    .expect("RunStopParameters should exist, this should never happen")
                    .collect_until,
            )?;
            hdf5.close()?;
        }
        Ok(())
    }

    pub(crate) fn abort_run(
        &mut self,
        nexus_settings: Option<&NexusSettings>,
        absolute_stop_time_ms: u64,
    ) -> NexusWriterResult<()> {
        self.parameters.set_aborted_run(absolute_stop_time_ms)?;

        if let Some(nexus_settings) = nexus_settings {
            let mut hdf5 =
                RunFile::open_runfile(nexus_settings.get_local_path(), &self.parameters.run_name)?;

            let collect_until = self
                .parameters
                .run_stop_parameters
                .as_ref()
                .expect("RunStopParameters should exist, this should never happen")
                .collect_until;

            hdf5.set_end_time(&collect_until)?;
            let relative_stop_time_ms =
                (collect_until - self.parameters.collect_from).num_milliseconds();
            hdf5.push_aborted_run_warning(
                relative_stop_time_ms,
                &self.parameters.collect_from,
                nexus_settings,
            )?;
            hdf5.close()?;
        }
        Ok(())
    }

    pub(crate) fn is_message_timestamp_valid(&self, timestamp: &NexusDateTime) -> bool {
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

impl SpannedAggregator for Run {
    fn span_init(&mut self) -> Result<(), SpanOnceError> {
        let span = info_span!(parent: None,
            "Run",
            "run_name" = self.parameters.run_name.as_str(),
            "instrument_name" = self.parameters.instrument_name.as_str(),
            "run_has_run_stop" = tracing::field::Empty
        );
        self.span_mut().init(span)
    }

    fn link_current_span<F: Fn() -> Span>(
        &self,
        aggregated_span_fn: F,
    ) -> Result<(), SpanOnceError> {
        self.span()
            .get()?
            .in_scope(aggregated_span_fn)
            .follows_from(tracing::Span::current());
        Ok(())
    }

    fn end_span(&self) -> Result<(), SpanOnceError> {
        self.span()
            .get()?
            .record("run_has_run_stop", self.has_run_stop());
        Ok(())
    }
}
