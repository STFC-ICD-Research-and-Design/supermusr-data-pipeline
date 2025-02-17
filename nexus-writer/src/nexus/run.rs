use super::{
    error::NexusWriterResult, hdf5_file::RunFile, NexusConfiguration, NexusDateTime, NexusSettings,
    RunParameters,
};
use chrono::{Duration, Utc};
use std::{fs::create_dir_all, future::Future, io, path::Path};
use supermusr_common::spanned::{SpanOnce, SpanOnceError, Spanned, SpannedAggregator, SpannedMut};
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::{info, info_span, warn, Span};

pub(crate) struct Run {
    span: SpanOnce,
    parameters: RunParameters,
}

impl Run {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn new_run(
        local_path: Option<&Path>,
        parameters: RunParameters,
        nexus_settings: &NexusSettings,
        nexus_configuration: &NexusConfiguration,
    ) -> NexusWriterResult<Self> {
        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::new_runfile(local_path, &parameters.run_name, nexus_settings)?;
            hdf5.init(&parameters, nexus_configuration)?;
            hdf5.close()?;
        }

        Ok(Self {
            span: Default::default(),
            parameters,
        })
    }

    pub(crate) fn resume_partial_run(
        local_path: &Path,
        filename: &str,
        nexus_settings: &NexusSettings,
    ) -> NexusWriterResult<Self> {
        let mut run = RunFile::open_runfile(local_path, filename)?;
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

    #[tracing::instrument(skip_all, level = "info")]
    pub(crate) fn move_to_archive(
        &self,
        local_name: &Path,
        archive_name: &Path,
    ) -> io::Result<impl Future<Output = ()>> {
        create_dir_all(archive_name)?;

        let from_path = RunParameters::get_hdf5_filename(local_name, &self.parameters.run_name);
        let to_path = RunParameters::get_hdf5_filename(archive_name, &self.parameters.run_name);

        let span = tracing::Span::current();
        let future = async move {
            info_span!(parent: &span, "move-async").in_scope(|| {
                match std::fs::copy(from_path.as_path(), to_path) {
                    Ok(bytes) => info!("File Move Succesful. {bytes} byte(s) moved."),
                    Err(e) => warn!("File Move Error {e}"),
                }
                if let Err(e) = std::fs::remove_file(from_path) {
                    warn!("Error removing temporary file: {e}");
                }
            });
        };
        Ok(future)
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_logdata_to_run(
        &mut self,
        local_path: Option<&Path>,
        logdata: &f144_LogData,
        nexus_settings: &NexusSettings,
    ) -> NexusWriterResult<()> {
        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::open_runfile(local_path, &self.parameters.run_name)?;
            hdf5.push_logdata_to_runfile(logdata, &self.parameters.collect_from, nexus_settings)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_alarm_to_run(
        &mut self,
        local_path: Option<&Path>,
        alarm: Alarm,
        nexus_settings: &NexusSettings,
    ) -> NexusWriterResult<()> {
        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::open_runfile(local_path, &self.parameters.run_name)?;
            hdf5.push_alarm_to_runfile(alarm, &self.parameters.collect_from, nexus_settings)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_selogdata(
        &mut self,
        local_path: Option<&Path>,
        logdata: se00_SampleEnvironmentData,
        nexus_settings: &NexusSettings,
    ) -> NexusWriterResult<()> {
        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::open_runfile(local_path, &self.parameters.run_name)?;
            hdf5.push_selogdata(logdata, &self.parameters.collect_from, nexus_settings)?;
            hdf5.close()?;
        }

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_frame_eventlist_message(
        &mut self,
        local_path: Option<&Path>,
        message: &FrameAssembledEventListMessage,
        nexus_settings: &NexusSettings,
    ) -> NexusWriterResult<()> {
        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::open_runfile(local_path, &self.parameters.run_name)?;
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
        local_path: Option<&Path>,
        absolute_stop_time_ms: u64,
        nexus_settings: &NexusSettings,
    ) -> NexusWriterResult<()> {
        self.parameters.set_aborted_run(absolute_stop_time_ms)?;

        if let Some(local_path) = local_path {
            let mut hdf5 = RunFile::open_runfile(local_path, &self.parameters.run_name)?;

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
