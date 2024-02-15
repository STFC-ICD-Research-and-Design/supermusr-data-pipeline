use super::{event_message::GenericEventMessage, run::Run, RunParameters};
use anyhow::{anyhow, Error, Result};
use chrono::{Duration, Utc};
use std::{path::PathBuf, collections::VecDeque};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
use tracing::warn;

#[derive(Default, Debug)]
pub(crate) struct Nexus {
    filename: PathBuf,
    run_cache: VecDeque<Run>,
    run_number: u32,
}

impl Nexus {
    pub(crate) fn new(filename: PathBuf) -> Self {
        Self {
            filename,
            ..Default::default()
        }
    }

    fn append_context(&self, e: Error) -> Error {
        anyhow!(
            "\nNexus Context: {0:?}\n{e}",
            self.run_cache
                .iter()
                .map(|run| run.get_name())
                .collect::<Vec<_>>()
        )
    }

    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> Result<()> {
        //  Check that the last run has already had its stop command
        if self
            .run_cache
            .back()
            .map(|run| run.has_run_stop())
            .unwrap_or(true)
        {
            let run = Run::new(&self.filename, RunParameters::new(data, self.run_number)?)?;
            self.run_cache.push_back(run);
            Ok(())
        } else {
            Err(self.append_context(anyhow!("Unexpected RunStart Command.")))
        }
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<()> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run
                .set_stop_if_valid(&self.filename, data)
                .map_err(|e| self.append_context(e))
        } else {
            Err(self.append_context(anyhow!("Unexpected RunStop Command")))
        }
    }

    pub(crate) fn process_message(&mut self, message: &GenericEventMessage<'_>) -> Result<()> {
        for run in &mut self.run_cache.iter_mut() {
            if run.is_message_timestamp_valid(&message.timestamp)? {
                run.push_message(&self.filename, message)?;
                return Ok(());
            }
        }
        warn!("No run found for message");
        Ok(())
    }

    pub(crate) fn write_files(&mut self, delay: &Duration) -> Result<()> {
        if let Some(run) = self.run_cache.front() {
            if run.is_ready_to_write(&Utc::now(), delay) {
                let run = self.run_cache.pop_front().unwrap(); // This will never panic
                run.close()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use supermusr_streaming_types::{
        ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, root_as_run_stop, RunStopArgs},
        ecs_pl72_run_start_generated::{finish_run_start_buffer, root_as_run_start, RunStartArgs},
        flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer},
    };

    use super::*;

    fn create_start<'a, 'b: 'a>(
        fbb: &'b mut FlatBufferBuilder,
        name: &str,
    ) -> Result<RunStart<'a>, InvalidFlatbuffer> {
        let args = RunStartArgs {
            start_time: 16,
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
    ) -> Result<RunStop<'a>, InvalidFlatbuffer> {
        let args = RunStopArgs {
            stop_time: 17,
            run_name: Some(fbb.create_string(name)),
            ..Default::default()
        };
        let message = RunStop::create(fbb, &args);
        finish_run_stop_buffer(fbb, message);
        root_as_run_stop(fbb.finished_data())
    }
    /*
    #[test]
    fn empty_run() {
        let mut nexus = Nexus::<EventList>::new();
        let mut fbb = FlatBufferBuilder::new();
        let start = create_start(&mut fbb, "Test1").unwrap();
        nexus.start_command(start).unwrap();

        assert_eq!(nexus.run_cache.len(), 1);
        assert_eq!(nexus.run_cache[0].collect_from, 16);
        assert!(nexus.run_cache[0].run_stop_parameters.is_none());

        fbb.reset();
        let stop = create_stop(&mut fbb, "Test1").unwrap();
        nexus.stop_command(stop).unwrap();

        assert!(nexus.run_cache[0].run_stop_parameters.is_some());
        assert_eq!(
            nexus.run_cache[0]
                .run_stop_parameters
                .as_ref()
                .unwrap()
                .collect_until,
            17
        );
    }

    #[test]
    fn no_run_start() {
        let mut nexus = Nexus::<EventList>::new();
        let mut fbb = FlatBufferBuilder::new();

        let stop = create_stop(&mut fbb, "Test1").unwrap();
        assert!(nexus.stop_command(stop).is_err());
    } */
}
