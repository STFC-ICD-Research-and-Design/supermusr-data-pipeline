use std::{fs::create_dir_all, path::PathBuf};

use super::{
    eventlist::EventList,
    writer::{add_new_field_to, add_new_group_to, add_new_string_field_to, set_group_nx_class},
};
use anyhow::Result;
use hdf5::{file::File, Group};

pub(crate) trait BuilderType: Default {
    type MessageType<'a>;

    fn process_message(&mut self, data: &Self::MessageType<'_>) -> Result<()>;
    fn write_hdf5(&self, parent: &Group) -> Result<()>;
}

#[derive(Default)]
pub(crate) struct Nexus<T: BuilderType> {
    run_number: usize,
    lists: T,
}

impl<T: BuilderType> Nexus<T> {
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

    fn write_header(&self, parent: &Group) -> Result<Group> {
        set_group_nx_class(parent, "NX_root")?;

        add_new_string_field_to(parent, "file_name", "My File Name", &[])?;
        add_new_string_field_to(parent, "file_time", "Now", &[])?;

        let entry = add_new_group_to(parent, "raw_data_1", "NXentry")?;

        add_new_field_to(&entry, "IDF_version", 2, &[])?;
        add_new_string_field_to(&entry, "definition", "muonTD", &[])?;
        add_new_field_to(&entry, "run_number", self.run_number, &[])?;
        add_new_string_field_to(&entry, "experiment_identifier", "", &[])?;
        add_new_string_field_to(&entry, "start_time", "", &[])?;
        add_new_string_field_to(&entry, "end_time", "", &[])?;

        add_new_group_to(&entry, "detector_1", "NXeventdata")
    }

    pub(crate) fn process_message(&mut self, data: &T::MessageType<'_>) -> Result<()> {
        self.lists.process_message(data)
    }
}

impl Nexus<EventList> {
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

    /*
    fn write_metadata(&mut self, data : &DigitizerAnalogTraceMessage) -> Result<()> {
        add_new_field_to("frame_number", data.metadata().frame_number())?;
        add_new_field_to("period_number", data.metadata().period_number())?;
        add_new_field_to("protons_per_pulse", data.metadata().protons_per_pulse())?;
        add_new_field_to("running", data.metadata().running())?;
        add_new_field_to("veto_flags", data.metadata().veto_flags())?;

        match data.metadata().timestamp() {
            Some(_) => {
                self.add_new_string_field_to("value", "Now")?;
            },
            None => ()
        }
        Ok(())
    }
    */
}
