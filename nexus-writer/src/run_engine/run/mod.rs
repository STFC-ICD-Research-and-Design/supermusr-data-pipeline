mod run_parameters;
mod run_spans;

use crate::{
    error::NexusWriterResult,
    nexus::{AlarmMessage, LogMessage, NexusFileInterface},
};

use super::{
    run_messages::{
        InitialiseNewNexusStructure, InternallyGeneratedLog, PushAlarm, PushFrameEventList,
        PushInternallyGeneratedLogWarning, PushRunLog, PushRunStart, PushSampleEnvironmentLog,
        SetEndTime, UpdatePeriodList,
    },
    NexusDateTime, NexusSettings, SampleEnvironmentLog,
};
use chrono::{Duration, Utc};
use std::{io, path::Path};
use supermusr_common::spanned::SpanOnce;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
};
use tracing::{error, info, info_span};

pub(crate) use run_parameters::{NexusConfiguration, RunParameters, RunStopParameters};
pub(crate) use run_spans::RunSpan;

pub(crate) struct Run<I: NexusFileInterface> {
    span: SpanOnce,
    parameters: RunParameters,
    file: I,
}

impl<I: NexusFileInterface> Run<I> {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn new_run(
        nexus_settings: &NexusSettings,
        run_start: RunStart,
        nexus_configuration: &NexusConfiguration,
    ) -> NexusWriterResult<Self> {
        let parameters = RunParameters::new(run_start)?;
        let file_path =
            RunParameters::get_hdf5_filename(nexus_settings.get_local_path(), &parameters.run_name);
        let mut file = I::build_new_file(&file_path, nexus_settings)?;

        file.handle_message(&InitialiseNewNexusStructure(
            &parameters,
            nexus_configuration,
        ))?;
        file.handle_message(&PushRunStart(run_start))?;
        file.flush()?;

        let mut run = Self {
            span: Default::default(),
            parameters,
            file,
        };
        run.link_run_start_span();

        Ok(run)
    }

    pub(crate) fn resume_partial_run(
        nexus_settings: &NexusSettings,
        filename: &str,
    ) -> NexusWriterResult<Self> {
        let file_path = RunParameters::get_hdf5_filename(nexus_settings.get_local_path(), filename);
        let mut file = I::open_from_file(&file_path)?;
        let parameters = file.extract_run_parameters()?;
        file.handle_message(&PushInternallyGeneratedLogWarning {
            message: InternallyGeneratedLog::RunResume {
                resume_time: &Utc::now(),
            },
            origin: &parameters.collect_from,
            settings: nexus_settings.get_chunk_sizes(),
        })?;
        file.flush()?;

        Ok(Self {
            span: Default::default(),
            parameters,
            file,
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
    pub(crate) fn push_frame_event_list(
        &mut self,
        nexus_settings: &NexusSettings,
        message: FrameAssembledEventListMessage,
    ) -> NexusWriterResult<()> {
        self.link_frame_event_list_span(message);
        self.file
            .handle_message(&PushFrameEventList { message: &message })?;

        if !self
            .parameters
            .periods
            .contains(&message.metadata().period_number())
        {
            self.parameters
                .periods
                .push(message.metadata().period_number());
            self.file.handle_message(&UpdatePeriodList {
                periods: &self.parameters.periods,
            })?;
        }

        if !message.complete() {
            self.file
                .handle_message(&PushInternallyGeneratedLogWarning {
                    message: InternallyGeneratedLog::IncompleteFrame { frame: &message },
                    origin: &self.parameters.collect_from,
                    settings: nexus_settings.get_chunk_sizes(),
                })?;
        }
        self.file.flush()?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_run_log(
        &mut self,
        nexus_settings: &NexusSettings,
        logdata: &f144_LogData,
    ) -> NexusWriterResult<()> {
        self.link_run_log_span();

        self.file.handle_message(&PushRunLog {
            runlog: &logdata.as_ref_with_origin(&self.parameters.collect_from),
            settings: nexus_settings.get_chunk_sizes(),
        })?;
        self.file.flush()?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_sample_environment_log(
        &mut self,
        nexus_settings: &NexusSettings,
        selog: &SampleEnvironmentLog,
    ) -> NexusWriterResult<()> {
        self.link_sample_environment_log_span();

        self.file.handle_message(&PushSampleEnvironmentLog {
            selog: &selog.as_ref_with_origin(&self.parameters.collect_from),
            settings: nexus_settings.get_chunk_sizes(),
        })?;
        self.file.flush()?;

        self.parameters.update_last_modified();
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn push_alarm(
        &mut self,
        nexus_settings: &NexusSettings,
        alarm: &Alarm,
    ) -> NexusWriterResult<()> {
        self.link_alarm_span();

        self.file.handle_message(&PushAlarm(
            &alarm.as_ref_with_origin(&self.parameters.collect_from),
            nexus_settings.get_chunk_sizes(),
        ))?;
        self.file.flush()?;

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

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    pub(crate) fn set_stop_if_valid(&mut self, data: &RunStop<'_>) -> NexusWriterResult<()> {
        self.link_run_stop_span();

        self.parameters.set_stop_if_valid(data)?;

        self.file.handle_message(&SetEndTime {
            end_time: &self
                .parameters
                .run_stop_parameters
                .as_ref()
                .expect("RunStopParameters should exist, this should never happen")
                .collect_until,
        })?;
        self.file.flush()?;
        Ok(())
    }

    pub(crate) fn abort_run(
        &mut self,
        nexus_settings: &NexusSettings,
        absolute_stop_time_ms: u64,
    ) -> NexusWriterResult<()> {
        self.parameters.set_aborted_run(absolute_stop_time_ms)?;

        let collect_until = self
            .parameters
            .run_stop_parameters
            .as_ref()
            .expect("RunStopParameters should exist, this should never happen")
            .collect_until;

        self.file.handle_message(&SetEndTime {
            end_time: &collect_until,
        })?;

        let relative_stop_time_ms =
            (collect_until - self.parameters.collect_from).num_milliseconds();
        self.file
            .handle_message(&PushInternallyGeneratedLogWarning {
                message: InternallyGeneratedLog::AbortRun {
                    stop_time_ms: relative_stop_time_ms,
                },
                origin: &self.parameters.collect_from,
                settings: nexus_settings.get_chunk_sizes(),
            })?;
        self.file.flush()?;

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

    pub(crate) fn close(self) {
        self.file.close();
    }
}
