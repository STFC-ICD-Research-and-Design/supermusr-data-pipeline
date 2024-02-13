use super::{
    messages::{InstanceType, ListType},
    RunParameters,
};
use anyhow::{anyhow, Result};
use chrono::{Duration, Utc};
use hdf5::file::File;
use std::{collections::VecDeque, fs::create_dir_all, path::Path};
use supermusr_streaming_types::{
    ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart,
};
use crate::hdf5_writer::{add_new_group_to, set_attribute_list_to, set_group_nx_class, Hdf5Writer};
use crate::nexus::nexus_class as NX;

use tracing::{debug, warn};

#[derive(Default,Debug)]
pub(crate) struct Nexus<L: ListType> {
    run_cache: VecDeque<RunParameters>,
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
            .map(|run| run.run_stop_parameters.is_some())
            .unwrap_or(true)
        {
            self.run_cache
                .push_back(RunParameters::new(data, self.run_number)?);
            Ok(())
        } else {
            Err(anyhow!(
                "Unexpected RunStart Command. Last Run: {0:?}",
                self.run_cache.back().unwrap()
            ))
        }
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<()> {
        if let Some(last_run) = self.run_cache.back_mut() {
            last_run.set_stop_if_valid(data)
        } else {
            Err(anyhow!("Unexpected RunStop Command"))
        }
    }

    pub(crate) fn process_message(
        &mut self,
        data: &<L::MessageInstance as InstanceType>::MessageType<'_>,
    ) -> Result<()> {
        let message_instance = L::MessageInstance::extract_message(data)?;
        self.lost_message_cache.push(message_instance);
        Ok(())
    }

    pub(crate) fn write_files(&mut self, filename: &Path, delay: &Duration) -> Result<()> {
        if let Some(run) = self.run_cache.front() {
            if run.is_ready_to_write(&Utc::now(), delay) {
                let run = self.run_cache.pop_front().unwrap(); // This will never panic
                let lists = self.collect_run_messages(&run)?;
                if lists.has_content() {
                    self.write_run_to_file(filename, &run, &lists)?;
                    self.run_number += 1;
    
                    debug!("Popped completed run, {0} runs remaining.",self.run_cache.len());
                } else {
                    self.run_cache.push_back(run);
                    warn!("Run has no content: returning to the queue.");
                }
            }
        }
        Ok(())
    }
}

impl<L: ListType> Nexus<L> {
    pub(super) fn collect_run_messages(
        &mut self,
        run: &RunParameters,
    ) -> Result<L> {
        debug!(
            "Collecting upto {0} lost messages to Run with start: {1} and stop: {2}",
            self.lost_message_cache.len(),
            run.collect_from,
            run.run_stop_parameters.as_ref().map(|rsp|rsp.collect_until).unwrap_or_default(),
        );

        let found_messages = self.lost_message_cache
            .iter()
            .map(|message| Ok(
                run.is_message_timestamp_valid(message.timestamp())?
                    .then_some(message)
            ))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .flatten();

        let mut lists = L::default();
        for message in found_messages {
            lists.append_message(message.clone())?;
        }

        // Note it is safe to call unwrap here as if any error were possible,
        // the method would already have returned
        self.lost_message_cache.retain(|message|
            !run.is_message_timestamp_valid(message.timestamp()).unwrap()
        );
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
