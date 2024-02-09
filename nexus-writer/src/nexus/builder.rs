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

#[derive(Default)]
pub(crate) struct Nexus<L: ListType> {
    start_time: Option<DateTime<Utc>>,
    run_cache: VecDeque<Run<L>>,
    lost_message_cache: Vec<L::MessageInstance>,
    run_number: usize,
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
            self.run_cache.push_back(Run::new(RunParameters::new(data)?));
            Ok(())
        } else {
            Err(anyhow!(
                "Unexpected RunStart Command. Last Run: {0:?}",
                self.run_cache.back().unwrap().parameters()
            ))
        }
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<()> {
        if let Some(run) = self.run_cache.back_mut() {
            run.parameters_mut().set_stop_if_valid(data)?;
            run.repatriate_lost_messsages(&mut self.lost_message_cache)
        } else {
            Err(anyhow!("Unexpected RunStop Command"))
        }
    }

    pub(crate) fn process_message(
        &mut self,
        data: &<L::MessageInstance as InstanceType>::MessageType<'_>,
    ) -> Result<()> {
        let instance = L::MessageInstance::extract_message(data)?;
        match self.run_cache.iter_mut().find(|run| {
            run.parameters()
                .is_message_timestamp_valid(instance.timestamp())
        }) {
            Some(run) => run.lists_mut().append_message(instance)?,
            None => self.lost_message_cache.push(instance),
        };
        Ok(())
    }

    pub(crate) fn write_files(&mut self, filename: &Path, delay: u64) -> Result<()> {
        if let Some(until) = self
            .run_cache
            .front()
            .and_then(|run| run.parameters().collect_until)
        {
            if Utc::now().timestamp_millis() > (until + delay) as i64 {
                if let Some(mut run) = self.run_cache.pop_front() {
                    run.repatriate_lost_messsages(&mut self.lost_message_cache)?;
                    log::debug!("Popped completed run, {0} runs remaining.", self.run_cache.len());
                    self.write_file(filename, &run)?;
                    self.run_number += 1;
                }
            }
        }
        Ok(())
    }

    fn write_file(&self, filename: &Path, run: &Run<L>) -> Result<()> {
        create_dir_all(filename)?;
        let filename = {
            let mut filename = filename.to_owned();
            filename.push(run.parameters().run_name.as_str());
            filename.set_extension("nxs");
            filename
        };
        log::debug!("Saving file {0}.", filename.display());
        let file = File::create(filename)?;
        run.write_hdf5(&file, self.run_number)?;
        Ok(file.close()?)
    }
}
