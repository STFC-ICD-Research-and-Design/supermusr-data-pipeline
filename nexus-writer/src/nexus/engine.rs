use super::{Run, RunParameters};
use chrono::{DateTime, Duration, Utc};
#[cfg(test)]
use std::collections::vec_deque;
use std::{
    collections::VecDeque,
    path::{Path, PathBuf},
};
use supermusr_common::spanned::SpannedAggregator;
use supermusr_streaming_types::{
    aev2_frame_assembled_event_v2_generated::FrameAssembledEventListMessage,
    ecs_6s4t_run_stop_generated::RunStop, ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::f144_LogData, ecs_pl72_run_start_generated::RunStart,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
};
use tracing::warn;

pub(crate) struct NexusEngine {
    filename: Option<PathBuf>,
    run_cache: VecDeque<Run>,
    run_number: u32,
    nexus_settings: NexusSettings,
}

impl NexusEngine {
    #[tracing::instrument(skip_all)]
    pub(crate) fn new(filename: Option<&Path>, nexus_settings: NexusSettings) -> Self {
        Self {
            filename: filename.map(ToOwned::to_owned),
            run_cache: Default::default(),
            run_number: 0,
            nexus_settings,
        }
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
    ) -> anyhow::Result<Option<&Run>> {
        let timestamp = DateTime::<Utc>::from_timestamp_nanos(data.packet_timestamp());
        if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_selogdata(self.filename.as_deref(), data, &self.nexus_settings)?;
            Ok(Some(run))
        } else {
            warn!("No run found for selogdata message with timestamp: {timestamp}");
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn logdata(&mut self, data: &f144_LogData<'_>) -> anyhow::Result<Option<&Run>> {
        let timestamp = DateTime::<Utc>::from_timestamp_nanos(data.timestamp());
        if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_logdata_to_run(self.filename.as_deref(), data, &self.nexus_settings)?;
            Ok(Some(run))
        } else {
            warn!("No run found for logdata message with timestamp: {timestamp}");
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn alarm(&mut self, data: Alarm<'_>) -> anyhow::Result<Option<&Run>> {
        let timestamp = DateTime::<Utc>::from_timestamp_nanos(data.timestamp());
        if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_alarm_to_run(self.filename.as_deref(), data)?;
            Ok(Some(run))
        } else {
            warn!("No run found for alarm message with timestamp: {timestamp}");
            Ok(None)
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> anyhow::Result<&mut Run> {
        //  Check that the last run has already had its stop command
        if self
            .run_cache
            .back()
            .map(|run| run.has_run_stop())
            .unwrap_or(true)
        {
            let mut run = Run::new_run(
                self.filename.as_deref(),
                RunParameters::new(data, self.run_number)?,
                &self.nexus_settings,
            )?;
            if let Err(e) = run.span_init() {
                warn!("Run span initiation failed {e}")
            }
            self.run_cache.push_back(run);
            Ok(self.run_cache.back_mut().expect("Run exists"))
        } else {
            Err(anyhow::anyhow!("Unexpected RunStart Command."))
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> anyhow::Result<&Run> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run.set_stop_if_valid(self.filename.as_deref(), data)?;

            Ok(last_run)
        } else {
            Err(anyhow::anyhow!("Unexpected RunStop Command"))
        }
    }

    #[tracing::instrument(skip_all,
        target = "otel",
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
    ) -> anyhow::Result<Option<&Run>> {
        let timestamp: DateTime<Utc> = (*message
            .metadata()
            .timestamp()
            .ok_or(anyhow::anyhow!("Message timestamp missing."))?)
        .try_into()?;

        let run: Option<&Run> = if let Some(run) = self
            .run_cache
            .iter_mut()
            .find(|run| run.is_message_timestamp_valid(&timestamp))
        {
            run.push_message(self.filename.as_deref(), message)?;
            Some(run)
        } else {
            warn!("No run found for message with timestamp: {timestamp}");
            None
        };
        Ok(run)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub(crate) fn flush(&mut self, delay: &Duration) {
        self.run_cache.retain(|run| {
            if run.has_completed(delay) {
                if let Err(e) = run.end_span() {
                    warn!("Run span drop failed {e}")
                }
                false
            } else {
                true
            }
        });
    }
}

#[cfg(test)]
mod test {
    use crate::nexus::NexusSettings;

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
    ) -> anyhow::Result<RunStart<'a>, InvalidFlatbuffer> {
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
    ) -> anyhow::Result<RunStop<'a>, InvalidFlatbuffer> {
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
    ) -> anyhow::Result<FrameAssembledEventListMessage<'a>, InvalidFlatbuffer> {
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
        let mut nexus = NexusEngine::new(None, NexusSettings::default());
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
        let mut nexus = NexusEngine::new(None, NexusSettings::default());
        let mut fbb = FlatBufferBuilder::new();

        let stop = create_stop(&mut fbb, "Test1", 0).unwrap();
        assert!(nexus.stop_command(stop).is_err());
    }

    #[test]
    fn no_run_stop() {
        let mut nexus = NexusEngine::new(None, NexusSettings::default());
        let mut fbb = FlatBufferBuilder::new();

        let start1 = create_start(&mut fbb, "Test1", 0).unwrap();
        nexus.start_command(start1).unwrap();

        fbb.reset();
        let start2 = create_start(&mut fbb, "Test2", 0).unwrap();
        assert!(nexus.start_command(start2).is_err());
    }

    #[test]
    fn frame_messages_correct() {
        let mut nexus = NexusEngine::new(None, NexusSettings::default());
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
}

#[derive(Default, Debug)]
pub(crate) struct NexusSettings {
    pub(crate) framelist_chunk_size: usize,
    pub(crate) eventlist_chunk_size: usize,
    pub(crate) periodlist_chunk_size: usize,
    pub(crate) runloglist_chunk_size: usize,
    pub(crate) seloglist_chunk_size: usize,
    pub(crate) alarmlist_chunk_size: usize,
}

impl NexusSettings {
    pub(crate) fn new(framelist_chunk_size: usize, eventlist_chunk_size: usize) -> Self {
        Self {
            framelist_chunk_size,
            eventlist_chunk_size,
            periodlist_chunk_size: 8,
            runloglist_chunk_size: 64,
            seloglist_chunk_size: 1024,
            alarmlist_chunk_size: 32,
        }
    }
}
