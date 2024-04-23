use super::{RunLike, RunParameters};
use crate::GenericEventMessage;
use anyhow::{anyhow, Error, Result};
use chrono::Duration;
#[cfg(test)]
use std::collections::vec_deque;
use std::{collections::VecDeque, path::PathBuf};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
use tracing::warn;

#[derive(Default, Debug)]
pub(crate) struct Nexus<R: RunLike> {
    filename: Option<PathBuf>,
    run_cache: VecDeque<R>,
    run_number: u32,
}

impl<R: RunLike> Nexus<R> {
    pub(crate) fn new(filename: Option<PathBuf>) -> Self {
        Self {
            filename,
            run_cache: VecDeque::default(),
            run_number: u32::default(),
        }
    }

    fn append_context(&self, e: Error) -> Error {
        anyhow!(
            "\nNexus Context: {0:?}\n{e}",
            self.run_cache
                .iter()
                .map(|run| run.as_ref().get_name())
                .collect::<Vec<_>>()
        )
    }

    #[cfg(test)]
    fn cache_iter(&self) -> vec_deque::Iter<'_, R> {
        self.run_cache.iter()
    }

    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> Result<&R> {
        //  Check that the last run has already had its stop command
        if self
            .run_cache
            .back()
            .map(|run| run.as_ref().has_run_stop())
            .unwrap_or(true)
        {
            let run = R::new(
                self.filename.as_deref(),
                RunParameters::new(data, self.run_number)?,
            )?;
            self.run_cache.push_back(run);
            Ok(self.run_cache.back().unwrap())
        } else {
            Err(self.append_context(anyhow!("Unexpected RunStart Command.")))
        }
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<&R> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run
                .as_mut()
                .set_stop_if_valid(self.filename.as_deref(), data)?;
            Ok(last_run)
        } else {
            Err(anyhow!("Unexpected RunStop Command"))
        }
    }

    pub(crate) fn process_message(
        &mut self,
        message: &GenericEventMessage<'_>,
    ) -> Result<Option<&R>> {
        for run in &mut self.run_cache.iter_mut() {
            if run
                .as_ref()
                .is_message_timestamp_valid(&message.timestamp)?
            {
                run.as_mut()
                    .push_message(self.filename.as_deref(), message)?;
                return Ok(Some(run));
            }
        }
        warn!("No run found for message");
        Ok(None)
    }

    pub(crate) fn flush(&mut self, delay: &Duration) -> Result<()> {
        self.run_cache
            .retain(|run| !run.as_ref().has_completed(delay));
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{
        event_message::{test::create_frame_assembled_message, GenericEventMessage},
        nexus::{Nexus, Run},
    };
    use chrono::{DateTime, Duration, Utc};
    use supermusr_streaming_types::{
        ecs_6s4t_run_stop_generated::{
            finish_run_stop_buffer, root_as_run_stop, RunStop, RunStopArgs,
        },
        ecs_pl72_run_start_generated::{
            finish_run_start_buffer, root_as_run_start, RunStart, RunStartArgs,
        },
        flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer},
        frame_metadata_v1_generated::GpsTime,
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

    #[test]
    fn empty_run() {
        let mut nexus = Nexus::<Run>::new(None);
        let mut fbb = FlatBufferBuilder::new();
        let start = create_start(&mut fbb, "Test1", 16).unwrap();
        nexus.start_command(start).unwrap();

        assert_eq!(nexus.run_cache.len(), 1);
        assert_eq!(
            nexus.run_cache[0].parameters().collect_from,
            DateTime::<Utc>::from_timestamp_millis(16).unwrap()
        );
        assert!(nexus.run_cache[0]
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
        let mut nexus = Nexus::<Run>::new(None);
        let mut fbb = FlatBufferBuilder::new();

        let stop = create_stop(&mut fbb, "Test1", 0).unwrap();
        assert!(nexus.stop_command(stop).is_err());
    }

    #[test]
    fn no_run_stop() {
        let mut nexus = Nexus::<Run>::new(None);
        let mut fbb = FlatBufferBuilder::new();

        let start1 = create_start(&mut fbb, "Test1", 0).unwrap();
        nexus.start_command(start1).unwrap();

        fbb.reset();
        let start2 = create_start(&mut fbb, "Test2", 0).unwrap();
        assert!(nexus.start_command(start2).is_err());
    }

    #[test]
    fn frame_messages_correct() {
        let mut nexus = Nexus::<Run>::new(None);
        let mut fbb = FlatBufferBuilder::new();

        let ts = GpsTime::new(0, 1, 0, 0, 16, 0, 0, 0);
        let ts_start: DateTime<Utc> = GpsTime::new(0, 1, 0, 0, 15, 0, 0, 0).try_into().unwrap();
        let ts_end: DateTime<Utc> = GpsTime::new(0, 1, 0, 0, 17, 0, 0, 0).try_into().unwrap();

        let start = create_start(&mut fbb, "Test1", ts_start.timestamp_millis() as u64).unwrap();
        nexus.start_command(start).unwrap();

        fbb.reset();
        let message = create_frame_assembled_message(&mut fbb, &ts).unwrap();
        let m1 = GenericEventMessage::from_frame_assembled_event_list_message(message).unwrap();
        nexus.process_message(&m1).unwrap();

        let mut fbb = FlatBufferBuilder::new(); //  Need to create a new instance as we use m1 later
        let stop = create_stop(&mut fbb, "Test1", ts_end.timestamp_millis() as u64).unwrap();
        nexus.stop_command(stop).unwrap();

        assert_eq!(nexus.cache_iter().len(), 1);

        let run = nexus.cache_iter().next();

        assert!(run
            .unwrap()
            .is_message_timestamp_valid(&m1.timestamp)
            .unwrap());

        nexus.flush(&Duration::zero()).unwrap();
        assert_eq!(nexus.cache_iter().len(), 0);
    }
}
