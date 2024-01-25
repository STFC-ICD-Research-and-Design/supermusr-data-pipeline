use super::{
    messages::{ListType},
    writer::{
        add_new_field_to, add_new_group_to, add_new_string_field_to, set_group_nx_class,
    }
};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use hdf5::{file::File, Group};
use supermusr_streaming_types::{ecs_6s4t_run_stop_generated::RunStop, ecs_pl72_run_start_generated::RunStart};
use std::{collections::{LinkedList, VecDeque}, fs::create_dir_all, path::PathBuf};

struct Run {

}

struct RunParameters {
    collect_from: u64,
    collect_until: Option<u64>,
    num_periods: u32,
    run_name: String,
    instrument_name: String,
}

impl RunParameters {
    fn new(data: RunStart<'_>) -> Result<Self> {
        Ok(Self {
            collect_from: data.start_time(),
            collect_until: None,
            num_periods: data.n_periods(),
            run_name: data.run_name().ok_or(anyhow!("Run Name not found"))?.to_owned(),
            instrument_name: data.instrument_name().ok_or(anyhow!("Instrument Name not found"))?.to_owned(),
        })
    }
    fn update_if_valid(&self, data: RunStart<'_>) -> Result<Self> {
        if let Some(until) = self.collect_until {
            if until <= data.start_time() {
                Self::new(data)
            } else {
                Err(anyhow!("New Start Time is earlier old Stop Time"))
            }
        } else {
            Err(anyhow!("New Start Command before Stop Command received."))
        }
    }
    fn set_stop_if_valid(&mut self, data: RunStop<'_>) -> Result<()> {
        if self.collect_until.is_some() {
            Err(anyhow!("Stop Command before Start Command"))
        } else {
            if self.collect_from < data.stop_time() {
                self.collect_until = Some(data.stop_time());
                Ok(())
            } else {
                Err(anyhow!("Stop Time earlier than current Start Time."))
            }
        }
    }

    fn is_message_timestamp_valid(&self, timestamp: &DateTime<Utc>) -> Result<()> {
        let milis = timestamp.timestamp_millis();
        assert!(milis > 0);
        if (milis as u64) < self.collect_from {
            //MessageTimestampStatus::BeforeRunStart
        } else {
            if let Some(until) = self.collect_until {
                if (timestamp.timestamp_millis() as u64) < until {
                    //MessageTimestampStatus::ValidRun
                } else {
                    //MessageTimestampStatus::AfterRunStop
                }
            } else {
                //MessageTimestampStatus::NoRunStop
            }
        }
        Ok(())
    }
}

#[derive(Default)]
pub(crate) struct Nexus<L: ListType> {
    start_time: Option<DateTime<Utc>>,
    runs : VecDeque<(RunParameters,L)>,
    lost_messages : LinkedList<L::MessageInstance>,
    parameters: Option<RunParameters>,
    run_number: usize,
}

impl<T: ListType> Nexus<T> {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn next_run(&mut self) {
        self.run_number += 1;
    }

    pub(crate) fn init(&mut self) -> Result<()> {
        self.lists = T::default();
        Ok(())
    }

    pub(crate) fn start_command(&mut self, data: RunStart<'_>) -> Result<()> {
        if let Some(params) = &self.parameters {
            self.parameters = Some(params.update_if_valid(data)?);
        } else {
            self.parameters = Some(RunParameters::new(data)?);
            self.start_time = Some(DateTime::<Utc>::UNIX_EPOCH + Duration::milliseconds(data.start_time() as i64));
        }
        Ok(())
    }

    pub(crate) fn stop_command(&mut self, data: RunStop<'_>) -> Result<()> {
        if let Some(params) = &mut self.parameters {
            params.set_stop_if_valid(data)
        } else {
            Err(anyhow!("Stop Command before Start Command received."))
        }
    }

    pub(crate) fn process_message(&mut self, data: &T::MessageType<'_>) -> Result<()> {
        if let Some(params) = &self.parameters {
            let timestamp = self.lists.extract_message_timestamp(data)?;
            match params.is_message_timestamp_valid(&timestamp) {
                MessageTimestampStatus::ValidRun => self.lists.process_message(timestamp, data),
                MessageTimestampStatus::NoRunStart => todo!(),
                MessageTimestampStatus::NoRunStop => todo!(),
                MessageTimestampStatus::BeforeRunStart => self.buffer_lists.process_message(timestamp, data),
                MessageTimestampStatus::AfterRunStop => todo!(),
            }
        } else {
            Err(anyhow!("Message before first Start Command received."))
        }
    }

    fn write_header(&self, parent: &Group) -> Result<Group> {
        if let Some(params) = &self.parameters {
            set_group_nx_class(parent, "NX_root")?;
    
            add_new_string_field_to(parent, "file_name", "My File Name", &[])?;
            add_new_string_field_to(parent, "file_time", "Now", &[])?;
    
            let entry = add_new_group_to(parent, params.run_name.as_str(), "NXentry")?;
    
            add_new_field_to(&entry, "IDF_version", 2, &[])?;
            add_new_string_field_to(&entry, "definition", "muonTD", &[])?;
            add_new_field_to(&entry, "run_number", self.run_number, &[])?;
            add_new_string_field_to(&entry, "experiment_identifier", "", &[])?;
            add_new_string_field_to(&entry, "start_time", self.start_time.ok_or(anyhow!("File start time not found."))?.to_string().as_str(), &[])?;
            add_new_string_field_to(&entry, "end_time", (DateTime::<Utc>::UNIX_EPOCH + Duration::milliseconds(params.collect_until.ok_or(anyhow!("File end time not found."))? as i64)).to_string().as_str(), &[])?;
            add_new_group_to(&entry, "detector_1", "NXeventdata")
        } else {
            Err(anyhow!("No run parameters set"))
        }
    }

    pub(crate) fn write_file(&self, filename: &PathBuf) -> Result<()> {
        create_dir_all(filename)?;
        let mut filename = filename.clone();
        filename.push(self.run_number.to_string());
        filename.set_extension("nxs");
        let file = File::create(filename)?;
        let detector = self.write_header(&file)?;
        self.lists.write_hdf5(&detector)?;
        Ok(file.close()?)
    }
}
