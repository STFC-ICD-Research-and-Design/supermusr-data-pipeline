use super::hdf5_writer::{add_new_group_to, set_attribute_list_to, set_group_nx_class, Hdf5Writer};
use super::run::Run;
use super::GenericEventMessage;
use super::{
    messages::{InstanceType, ListType},
    RunParameters,
};
use crate::nexus::nexus_class as NX;
use anyhow::{anyhow, Error, Result};
use chrono::{Duration, Utc};
use hdf5::file::File;
use std::path::PathBuf;
use std::{collections::VecDeque, fs::create_dir_all, path::Path};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

use tracing::{debug, warn};

#[derive(Default, Debug)]
pub(crate) struct Nexus<L: ListType> {
    filename: PathBuf,
    run_cache: VecDeque<Run>,
    lost_message_cache: Vec<L::MessageInstance>,
    run_number: u32,
}

impl<L: ListType> Nexus<L> {
    pub(crate) fn new(filename: &Path) -> Self {
        Self {
            filename: filename.to_owned(),
            ..Default::default()
        }
    }

    fn append_context(&self, e: Error) -> Error {
        anyhow!(
            "\nNexus Context: {0:?}\n{e}",
            self.run_cache
                .iter()
                .map(|run| &run.run_parameters.run_name)
                .collect::<Vec<_>>()
        )
    }

    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> Result<()> {
        if self
            .run_cache
            .back()
            .map(|run| run.run_parameters.run_stop_parameters.is_some())
            .unwrap_or(true)
        {
            let mut run = Run::new(&self.filename, RunParameters::new(data, self.run_number)?)?;
            run.init().unwrap();
            self.run_cache.push_back(run);
            Ok(())
        } else {
            Err(self.append_context(anyhow!("Unexpected RunStart Command.")))
        }
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<()> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run
                .run_parameters
                .set_stop_if_valid(data)
                .map_err(|e| self.append_context(e))
        } else {
            Err(anyhow!("Unexpected RunStop Command"))
        }
    }

    pub(crate) fn process_message(
        &mut self,
        message: &GenericEventMessage<'_>,
    ) -> Result<()> {
        for run in &mut self.run_cache.iter_mut() {
            if run.run_parameters.is_message_timestamp_valid(&message.timestamp)? {
                run.push_message(message)?;
                return Ok(());
            }
        }
        warn!("No run found for message");
        Ok(())
    }

    pub(crate) fn write_files(&mut self, delay: &Duration) -> Result<()> {
        if let Some(run) = self.run_cache.front() {
            if run.run_parameters.is_ready_to_write(&Utc::now(), delay) {
                let run = self.run_cache.pop_front().unwrap(); // This will never panic
                run.close()?;
                /*
                //let lists = self.collect_run_messages(&run)?;
                if lists.has_content() {
                    self.write_run_to_file(filename, &run, &lists)?;
                    self.run_number += 1;

                    debug!(
                        "Popped completed run, {0} runs remaining.",
                        self.run_cache.len()
                    );
                } else {
                    self.run_cache.push_back(run);
                    warn!("Run has no content: returning to the queue.");
                } */
            }
        }
        Ok(())
    }
}

impl<L: ListType> Nexus<L> {
    pub(super) fn collect_run_messages(&mut self, run: &RunParameters) -> Result<L> {
        debug!(
            "Collecting upto {0} lost messages to Run with start: {1} and stop: {2}",
            self.lost_message_cache.len(),
            run.collect_from,
            run.run_stop_parameters
                .as_ref()
                .map(|rsp| rsp.collect_until)
                .unwrap_or_default(),
        );

        let found_messages = self
            .lost_message_cache
            .iter()
            .map(|message| {
                Ok(run
                    .is_message_timestamp_valid(message.timestamp())?
                    .then_some(message))
            })
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten();

        let mut lists = L::default();
        for message in found_messages {
            lists.append_message(message.clone())?;
        }

        // Note it is safe to call unwrap here as if any error were possible,
        // the method would already have returned
        self.lost_message_cache
            .retain(|message| !run.is_message_timestamp_valid(message.timestamp()).unwrap());
        debug!("{0} lost messages remaining", self.lost_message_cache.len());
        Ok(lists)
    }

    fn write_run_to_file(&self, filename: &Path, run: &RunParameters, lists: &L) -> Result<()> {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run.run_name.as_str());
            filename.set_extension("nxs");
            filename
        };
        debug!("File save begin. File: {0}.", filename.display());

        let file = File::create(filename)?;
        set_group_nx_class(&file, NX::ROOT)?;

        set_attribute_list_to(
            &file,
            &[
                ("HDF5_version", "1.14.3"), // Can this be taken directly from the nix package?
                ("NeXus_version", ""),      // Where does this come from?
                ("file_name", &file.filename()), //  This should be absolutized at some point
                ("file_time", Utc::now().to_string().as_str()), //  This should be formatted, the nanoseconds are overkill.
            ],
        )?;

        let entry = add_new_group_to(&file, "raw_data_1", NX::ENTRY)?;
        run.write_hdf5(&entry)?;

        let event_data = add_new_group_to(&entry, "detector_1", NX::EVENT_DATA)?;
        lists.write_hdf5(&event_data)?;

        debug!("File save end.");
        Ok(file.close()?)
    }
}

#[cfg(test)]
mod test {
    use supermusr_streaming_types::{
        ecs_6s4t_run_stop_generated::{finish_run_stop_buffer, root_as_run_stop, RunStopArgs},
        ecs_pl72_run_start_generated::{finish_run_start_buffer, root_as_run_start, RunStartArgs},
        flatbuffers::{FlatBufferBuilder, InvalidFlatbuffer},
    };

    use crate::nexus::EventList;

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
