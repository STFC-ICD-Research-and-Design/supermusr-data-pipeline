use crate::hdf5_writer::Hdf5Writer;

use super::{
    messages::{InstanceType, ListType},
    Run, RunParameters,
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::file::File;
use std::{collections::VecDeque, fs::create_dir_all, path::Path};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};

use tracing::{debug, warn};

#[derive(Default)]
pub(crate) struct Nexus<L: ListType> {
    start_time: Option<DateTime<Utc>>,
    run_cache: VecDeque<Run<L>>,
    lost_message_cache: Vec<L::MessageInstance>,
    run_number: u32,
}

impl<L: ListType> Nexus<L> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> Result<()> {
        if self
            .run_cache
            .back()
            .map(|run| run.parameters().collect_until.is_some())
            .unwrap_or(true)
        {
            if self.start_time.is_none() {
                self.start_time = Some(
                    DateTime::<Utc>::UNIX_EPOCH + Duration::milliseconds(data.start_time() as i64),
                );
            }
            self.run_cache
                .push_back(Run::new(RunParameters::new(data, self.run_number)?));
            Ok(())
        } else {
            Err(anyhow!(
                "Unexpected RunStart Command. Last Run: {0:?}",
                self.run_cache.back().unwrap().parameters()
            ))
        }
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<()> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run.parameters_mut().set_stop_if_valid(data)?;
            last_run.repatriate_lost_messsages(&mut self.lost_message_cache)
        } else {
            Err(anyhow!("Unexpected RunStop Command"))
        }
    }

    pub(crate) fn process_message(
        &mut self,
        data: &<L::MessageInstance as InstanceType>::MessageType<'_>,
    ) -> Result<()> {
        let message_instance = L::MessageInstance::extract_message(data)?;

        debug!("Finding Run that Message belongs to");
        //  Find the run to which this message exists
        let mut valid_runs = self.run_cache
            .iter_mut()
            .map(|run| Ok(
                run.is_message_timestamp_valid(message_instance.timestamp())?
                    .then_some(run)
            ))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten();
        
        match valid_runs.next() {
            Some(run) => {
                debug!("Found Message Run.");
                run.lists_mut().append_message(message_instance)?
            },
            None => {
                debug!("No valid message run found: adding to lost messages.");
                self.lost_message_cache.push(message_instance)
            },
        };

        //  There should be no more than one valid run
        if valid_runs.next().is_some() {
            warn!("Run times overlap detected.");
        }

        Ok(())
    }

    pub(crate) fn write_files(&mut self, filename: &Path, delay: u64) -> Result<()> {
        // If there is a run in the cache vector, and the first one
        // has a collect_until set, then retrieve it.
        if let Some(until) = self
            .run_cache
            .front()
            .and_then(|run|run.parameters().collect_until)
        {
            // If the time is at least `delay` ms passed `until`
            if Utc::now().timestamp_millis() > (until + delay) as i64 {
                let mut run = self.run_cache.pop_front().unwrap(); // This will never panic

                //  Gather any lost messages
                run.repatriate_lost_messsages(&mut self.lost_message_cache)?;

                debug!("Popped completed run, {0} runs remaining.",self.run_cache.len());

                self.write_run_to_file(filename, &run)?;
                self.run_number += 1;
            }
        }
        Ok(())
    }
}

impl<L: ListType> Nexus<L> {
    fn write_run_to_file(&self, filename: &Path, run: &Run<L>) -> Result<()> {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run.parameters().run_name.as_str());
            filename.set_extension("nxs");
            filename
        };
        debug!("Saving file {0}.", filename.display());
        let file = File::create(filename)?;
        run.write_hdf5(&file)?;
        Ok(file.close()?)
    }
}
