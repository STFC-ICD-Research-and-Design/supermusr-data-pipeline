use super::{
    error::{ErrorCodeLocation, FlatBufferMissingError, NexusWriterError, NexusWriterResult},
    NexusDateTime, Run, RunParameters,
};
use chrono::Duration;
use glob::glob;
#[cfg(test)]
use std::collections::vec_deque;
use std::{
    collections::VecDeque,
    ffi::OsStr,
    path::{Path, PathBuf},
};
use supermusr_common::spanned::SpannedAggregator;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::{info_span, warn};

pub(crate) struct NexusEngine {
    local_path: Option<PathBuf>,
    run_cache: VecDeque<Run>,
    run_number: u32,
    nexus_settings: NexusSettings,
    nexus_configuration: NexusConfiguration,
    run_move_cache: Vec<Run>,
}

impl NexusEngine {
    #[tracing::instrument(skip_all)]
    pub(crate) fn new(
        local_path: Option<&Path>,
        nexus_settings: NexusSettings,
        nexus_configuration: NexusConfiguration,
    ) -> Self {
        Self {
            local_path: local_path.map(ToOwned::to_owned),
            run_cache: Default::default(),
            run_number: 0,
            nexus_settings,
            nexus_configuration,
            run_move_cache: Default::default(),
        }
    }

    pub(crate) fn resume_partial_runs(&mut self) -> NexusWriterResult<()> {
        if let Some(local_path) = &self.local_path {
            let local_path_str = local_path.as_os_str().to_str().ok_or_else(|| {
                NexusWriterError::CannotConvertPath {
                    path: local_path.clone(),
                    location: ErrorCodeLocation::ResumePartialRunsLocalDirectoryPath,
                }
            })?;

            for filename in glob(&format!("{local_path_str}/*.nxs"))? {
                let filename = filename?;
                let filename_str =
                    filename
                        .file_stem()
                        .and_then(OsStr::to_str)
                        .ok_or_else(|| NexusWriterError::CannotConvertPath {
                            path: filename.clone(),
                            location: ErrorCodeLocation::ResumePartialRunsFilePath,
                        })?;
                let mut run = info_span!(
                    "Partial Run Found",
                    path = local_path_str,
                    file_name = filename_str
                )
                .in_scope(|| {
                    Run::resume_partial_run(local_path, filename_str, &self.nexus_settings)
                })?;
                if let Err(e) = run.span_init() {
                    warn!("Run span initiation failed {e}")
                }
                self.run_cache.push_back(run);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    fn cache_iter(&self) -> vec_deque::Iter<'_, Run> {
        self.run_cache.iter()
    }

    pub(crate) fn get_num_cached_runs(&self) -> usize {
        self.run_cache.len()
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn sample_envionment(
        &mut self,
        data: se00_SampleEnvironmentData<'_>,
    ) -> NexusWriterResult<Option<&Run>> {
        let timestamp = NexusDateTime::from_timestamp_nanos(data.packet_timestamp());
        if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_selogdata(self.local_path.as_deref(), data, &self.nexus_settings)?;
            Ok(Some(run))
        } else {
            warn!("No run found for selogdata message with timestamp: {timestamp}");
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn logdata(&mut self, data: &f144_LogData<'_>) -> NexusWriterResult<Option<&Run>> {
        let timestamp = NexusDateTime::from_timestamp_nanos(data.timestamp());
        if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_logdata_to_run(self.local_path.as_deref(), data, &self.nexus_settings)?;
            Ok(Some(run))
        } else {
            warn!("No run found for logdata message with timestamp: {timestamp}");
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn alarm(&mut self, data: Alarm<'_>) -> NexusWriterResult<Option<&Run>> {
        let timestamp = NexusDateTime::from_timestamp_nanos(data.timestamp());
        if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_alarm_to_run(self.local_path.as_deref(), data, &self.nexus_settings)?;
            Ok(Some(run))
        } else {
            warn!("No run found for alarm message with timestamp: {timestamp}");
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> NexusWriterResult<&mut Run> {
        //  If a run is already in progress, and is missing a run-stop
        //  then call an abort run on the current run.
        if self.run_cache.back().is_some_and(|run| !run.has_run_stop()) {
            self.abort_back_run(&data)?;
        }

        let mut run = Run::new_run(
            self.local_path.as_deref(),
            RunParameters::new(data, self.run_number)?,
            &self.nexus_settings,
            &self.nexus_configuration,
        )?;
        if let Err(e) = run.span_init() {
            warn!("Run span initiation failed {e}")
        }
        self.run_cache.push_back(run);
        Ok(self.run_cache.back_mut().expect("Run exists"))
    }

    #[tracing::instrument(skip_all, level = "warn", err(level = "warn")
        fields(
            run_name = data.run_name(),
            instrument_name = data.instrument_name(),
            start_time = data.start_time(),
        )
    )]
    fn abort_back_run(&mut self, data: &RunStart<'_>) -> NexusWriterResult<()> {
        self.run_cache
            .back_mut()
            .expect("run_cache::back_mut should exist")
            .abort_run(
                self.local_path.as_deref(),
                data.start_time(),
                &self.nexus_settings,
            )?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> NexusWriterResult<&Run> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run.set_stop_if_valid(self.local_path.as_deref(), data)?;

            Ok(last_run)
        } else {
            Err(NexusWriterError::UnexpectedRunStop(
                ErrorCodeLocation::StopCommand,
            ))
        }
    }

    #[tracing::instrument(skip_all,
        fields(
            metadata_timestamp = tracing::field::Empty,
            metadata_frame_number = message.metadata().frame_number(),
            metadata_period_number = message.metadata().period_number(),
            metadata_veto_flags = message.metadata().veto_flags(),
            metadata_protons_per_pulse = message.metadata().protons_per_pulse(),
            metadata_running = message.metadata().running()
        )
    )]
    pub(crate) fn process_event_list(
        &mut self,
        message: &FrameAssembledEventListMessage<'_>,
    ) -> NexusWriterResult<Option<&Run>> {
        let timestamp: NexusDateTime =
            (*message
                .metadata()
                .timestamp()
                .ok_or(NexusWriterError::FlatBufferMissing(
                    FlatBufferMissingError::Timestamp,
                    ErrorCodeLocation::ProcessEventList,
                ))?)
            .try_into()?;

        let run: Option<&Run> = if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_frame_eventlist_message(
                self.local_path.as_deref(),
                message,
                &self.nexus_settings,
            )?;
            Some(run)
        } else {
            warn!("No run found for message with timestamp: {timestamp}");
            None
        };
        Ok(run)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn flush(&mut self, delay: &Duration) {
        // Moves the runs into a new vector, then consumes it,
        // directing completed runs to self.run_move_cache
        // and incomplete ones back to self.run_cache
        let temp: Vec<_> = self.run_cache.drain(..).collect();
        for run in temp.into_iter() {
            if run.has_completed(delay) {
                if let Err(e) = run.end_span() {
                    warn!("Run span drop failed {e}")
                }
                self.run_move_cache.push(run);
            } else {
                self.run_cache.push_back(run);
            }
        }
    }

    /// If an additional archive location is set by the user,
    /// then completed runs placed in the vector `self.run_move_cache`
    /// have their nexus files asynchonously moved to that location.
    /// Afterwhich the runs are dropped.
    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) async fn flush_move_cache(&mut self) {
        if let Some((local_path, archive_path)) = Option::zip(
            self.local_path.as_deref(),
            self.nexus_settings.archive_path.as_deref(),
        ) {
            for run in self.run_move_cache.iter() {
                match run.move_to_archive(local_path, archive_path) {
                    Ok(move_to_archive) => move_to_archive.await,
                    Err(e) => warn!("Error Moving to Archive {e}"),
                }
            }
        }
        self.run_move_cache.clear();
    }

    #[tracing::instrument(skip_all, level = "info", name = "Closing", fields(num_runs_to_archive = self.run_move_cache.len()))]
    pub(crate) async fn close(mut self) {
        self.flush_move_cache().await;
    }
}

#[cfg(test)]
mod test {
    use crate::nexus::{NexusConfiguration, NexusSettings};

    use super::NexusEngine;
    use chrono::{DateTime, Duration, Utc};
    use supermusr_streaming_types::{
        aev2_frame_assembled_event_v2_generated::{
            finish_frame_assembled_event_list_message_buffer,
            root_as_frame_assembled_event_list_message, FrameAssembledEventListMessage,
            FrameAssembledEventListMessageArgs,
        },
        ecs_6s4t_run_stop_generated::{
            finish_run_stop_buffer, root_as_run_stop, RunStop, RunStopArgs,
        },
        ecs_pl72_run_start_generated::{
            finish_run_start_buffer, root_as_run_start, RunStart, RunStartArgs,
        },
        flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer},
        frame_metadata_v2_generated::{FrameMetadataV2, FrameMetadataV2Args, GpsTime},
    };

    fn create_start<'a, 'b: 'a>(
        fbb: &'b mut FlatBufferBuilder,
        name: &str,
        start_time: u64,
    ) -> Result<RunStart<'a>, InvalidFlatbuffer> {
        let args = RunStartArgs {
            start_time,
            run_name: Some(fbb.create_string(name)),
            instrument_name: Some(fbb.create_string("Super MuSR")),
            ..Default::default()
        };
        let message = RunStart::create(fbb, &args);
        finish_run_start_buffer(fbb, message);
        root_as_run_start(fbb.finished_data())
    }

    fn create_stop<'a, 'b: 'a>(
        fbb: &'b mut FlatBufferBuilder,
        name: &str,
        stop_time: u64,
    ) -> Result<RunStop<'a>, InvalidFlatbuffer> {
        let args = RunStopArgs {
            stop_time,
            run_name: Some(fbb.create_string(name)),
            ..Default::default()
        };
        let message = RunStop::create(fbb, &args);
        finish_run_stop_buffer(fbb, message);
        root_as_run_stop(fbb.finished_data())
    }

    fn create_metadata(timestamp: &GpsTime) -> FrameMetadataV2Args<'_> {
        FrameMetadataV2Args {
            timestamp: Some(timestamp),
            period_number: 0,
            protons_per_pulse: 0,
            running: false,
            frame_number: 0,
            veto_flags: 0,
        }
    }

    fn create_frame_assembled_message<'a, 'b: 'a>(
        fbb: &'b mut FlatBufferBuilder,
        timestamp: &GpsTime,
    ) -> Result<FrameAssembledEventListMessage<'a>, InvalidFlatbuffer> {
        let metadata = FrameMetadataV2::create(fbb, &create_metadata(timestamp));
        let args = FrameAssembledEventListMessageArgs {
            metadata: Some(metadata),
            ..Default::default()
        };
        let message = FrameAssembledEventListMessage::create(fbb, &args);
        finish_frame_assembled_event_list_message_buffer(fbb, message);
        root_as_frame_assembled_event_list_message(fbb.finished_data())
    }

    #[test]
    fn empty_run() {
        let mut nexus = NexusEngine::new(
            None,
            NexusSettings::default(),
            NexusConfiguration::new(None),
        );
        let mut fbb = FlatBufferBuilder::new();
        let start = create_start(&mut fbb, "Test1", 16).unwrap();
        nexus.start_command(start).unwrap();

        assert_eq!(nexus.run_cache.len(), 1);
        assert_eq!(
            nexus.run_cache.front().unwrap().parameters().collect_from,
            DateTime::<Utc>::from_timestamp_millis(16).unwrap()
        );
        assert!(nexus
            .run_cache
            .front()
            .unwrap()
            .parameters()
            .run_stop_parameters
            .is_none());

        fbb.reset();
        let stop = create_stop(&mut fbb, "Test1", 17).unwrap();
        nexus.stop_command(stop).unwrap();

        assert_eq!(nexus.cache_iter().len(), 1);

        let run = nexus.cache_iter().next();

        assert!(run.is_some());
        assert!(run.unwrap().parameters().run_stop_parameters.is_some());
        assert_eq!(run.unwrap().get_name(), "Test1");

        assert!(run.unwrap().parameters().run_stop_parameters.is_some());
        assert_eq!(
            run.unwrap()
                .parameters()
                .run_stop_parameters
                .as_ref()
                .unwrap()
                .collect_until,
            DateTime::<Utc>::from_timestamp_millis(17).unwrap()
        );
    }

    #[test]
    fn no_run_start() {
        let mut nexus = NexusEngine::new(
            None,
            NexusSettings::default(),
            NexusConfiguration::new(None),
        );
        let mut fbb = FlatBufferBuilder::new();

        let stop = create_stop(&mut fbb, "Test1", 0).unwrap();
        assert!(nexus.stop_command(stop).is_err());
    }

    #[test]
    fn no_run_stop() {
        let mut nexus = NexusEngine::new(
            None,
            NexusSettings::default(),
            NexusConfiguration::new(None),
        );
        let mut fbb = FlatBufferBuilder::new();

        let start1 = create_start(&mut fbb, "Test1", 0).unwrap();
        nexus.start_command(start1).unwrap();
        assert_eq!(nexus.get_num_cached_runs(), 1);

        fbb.reset();
        let start2 = create_start(&mut fbb, "Test2", 0).unwrap();
        nexus.start_command(start2).unwrap();
        assert_eq!(nexus.get_num_cached_runs(), 2);
    }

    #[test]
    fn frame_messages_correct() {
        let mut nexus = NexusEngine::new(
            None,
            NexusSettings::default(),
            NexusConfiguration::new(None),
        );
        let mut fbb = FlatBufferBuilder::new();

        let ts = GpsTime::new(0, 1, 0, 0, 16, 0, 0, 0);
        let ts_start: DateTime<Utc> = GpsTime::new(0, 1, 0, 0, 15, 0, 0, 0).try_into().unwrap();
        let ts_end: DateTime<Utc> = GpsTime::new(0, 1, 0, 0, 17, 0, 0, 0).try_into().unwrap();

        let start = create_start(&mut fbb, "Test1", ts_start.timestamp_millis() as u64).unwrap();
        nexus.start_command(start).unwrap();

        fbb.reset();
        let message = create_frame_assembled_message(&mut fbb, &ts).unwrap();
        nexus.process_event_list(&message).unwrap();

        let mut fbb = FlatBufferBuilder::new(); //  Need to create a new instance as we use m1 later
        let stop = create_stop(&mut fbb, "Test1", ts_end.timestamp_millis() as u64).unwrap();
        nexus.stop_command(stop).unwrap();

        assert_eq!(nexus.cache_iter().len(), 1);

        let run = nexus.cache_iter().next();

        let timestamp: DateTime<Utc> = (*message.metadata().timestamp().unwrap())
            .try_into()
            .unwrap();

        assert!(run.unwrap().is_message_timestamp_valid(&timestamp));

        nexus.flush(&Duration::zero());
        assert_eq!(nexus.cache_iter().len(), 0);
    }

    #[test]
    fn two_runs_flushed() {
        let mut nexus = NexusEngine::new(
            None,
            NexusSettings::default(),
            NexusConfiguration::new(None),
        );
        let mut fbb = FlatBufferBuilder::new();

        let ts_start: DateTime<Utc> = GpsTime::new(0, 1, 0, 0, 15, 0, 0, 0).try_into().unwrap();
        let ts_end: DateTime<Utc> = GpsTime::new(0, 1, 0, 0, 17, 0, 0, 0).try_into().unwrap();

        let start = create_start(&mut fbb, "TestRun1", ts_start.timestamp_millis() as u64).unwrap();
        nexus.start_command(start).unwrap();

        fbb.reset();
        let stop = create_stop(&mut fbb, "TestRun1", ts_end.timestamp_millis() as u64).unwrap();
        nexus.stop_command(stop).unwrap();

        assert_eq!(nexus.cache_iter().len(), 1);

        fbb.reset();
        let start = create_start(&mut fbb, "TestRun2", ts_start.timestamp_millis() as u64).unwrap();
        nexus.start_command(start).unwrap();

        fbb.reset();
        let stop = create_stop(&mut fbb, "TestRun2", ts_end.timestamp_millis() as u64).unwrap();
        nexus.stop_command(stop).unwrap();

        assert_eq!(nexus.cache_iter().len(), 2);

        nexus.flush(&Duration::zero());
        assert_eq!(nexus.cache_iter().len(), 0);
    }
}

#[derive(Default, Debug)]
pub(crate) struct NexusSettings {
    pub(crate) framelist_chunk_size: usize,
    pub(crate) eventlist_chunk_size: usize,
    pub(crate) periodlist_chunk_size: usize,
    pub(crate) runloglist_chunk_size: usize,
    pub(crate) seloglist_chunk_size: usize,
    pub(crate) alarmlist_chunk_size: usize,
    archive_path: Option<PathBuf>,
}

impl NexusSettings {
    pub(crate) fn new(
        framelist_chunk_size: usize,
        eventlist_chunk_size: usize,
        archive_path: Option<&Path>,
    ) -> Self {
        Self {
            framelist_chunk_size,
            eventlist_chunk_size,
            periodlist_chunk_size: 8,
            runloglist_chunk_size: 64,
            seloglist_chunk_size: 1024,
            alarmlist_chunk_size: 32,
            archive_path: archive_path.map(Path::to_owned),
        }
    }
}

#[derive(Clone, Default, Debug)]
pub(crate) struct NexusConfiguration {
    pub(crate) configuration: String,
}

impl NexusConfiguration {
    pub(crate) fn new(configuration: Option<String>) -> Self {
        Self {
            configuration: configuration.unwrap_or_default(),
        }
    }
}
