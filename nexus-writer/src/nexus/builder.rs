
use std::{path::{PathBuf, Path}, collections::HashMap};

use hdf5::{file::File, H5Type, Extents, Group, SimpleExtents};
use anyhow::{anyhow, Result};
use supermusr_common::{Time, Intensity, Channel};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;


use super::{add_new_group_to, add_new_string_field_to, add_new_field_to,set_group_nx_class, add_new_slice_field_to};

#[derive(Default)]
struct EventList {
    event_time_offset : Vec<Time>,
    pulse_height : Vec<Intensity>,
    event_time_zero : Vec<Time>,
    event_id : Vec<Channel>
}

#[derive(Default)]
struct HistogramList {
    
}
#[derive(Default)]
pub(crate) struct Nexus<T : Default> {
    run_number: usize,
    lists : T,
}

impl<T : Default> Nexus<T> {
    pub(crate) fn new () -> Self {
        Self::default()
    }
    pub(crate) fn init (&mut self) -> Result<()> {
        self.lists = T::default();
        Ok(())
    }

    // Header and 
    fn begin_entry(&mut self, parent : &Group) -> Result<()> {


        Ok(())
    }
    
    fn add_metadata_group (&mut self, data : &DigitizerAnalogTraceMessage) -> Result<()> {
        self.set_cur_path(&PathBuf::from(&format!("/NXroot/NXentry/detector_1/traces/trace_{0}/metadata", self.num_traces)));
        self.add_field("frame_number", data.metadata().frame_number())?;
        self.add_field("period_number", data.metadata().period_number())?;
        self.add_field("protons_per_pulse", data.metadata().protons_per_pulse())?;
        self.add_field("running", data.metadata().running())?;
        self.add_field("veto_flags", data.metadata().veto_flags())?;
        
        self.set_cur_path(&PathBuf::from(&format!("/NXroot/NXentry/detector_1/traces/trace_{0}/metadata/timestamp", self.num_traces)));
        match data.metadata().timestamp() {
            Some(_) => {
                self.add_new_string_field_to("value", "Now")?;
            },
            None => ()
        }
        Ok(())
    }

    fn write_header(&self, parent : &Group) -> Result<Group>  {
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
}

impl Nexus<EventList> {
    pub(crate) fn add_event_group (&mut self, data : &DigitizerEventListMessage) -> Result<()> {
        self.lists.pulse_height.extend(data.voltage().unwrap().iter());
        self.lists.event_time_zero.extend(data.time().unwrap().iter());
        self.lists.event_id.extend(data.channel().unwrap().iter());
        Ok(())
    }

    pub(crate) fn write_file(&self, filename : &PathBuf) -> Result<()> {
        let file = File::create(filename)?;
        let detector = self.write_header(&file)?;
        self.write_events(&detector);
        Ok(file.close()?)
    }
        
    fn write_events(&self, detector : &Group) -> Result<()> {
        add_new_slice_field_to(&detector, "pulse_height", &self.lists.pulse_height, &[("units", "mV")])?;
        add_new_slice_field_to(&detector, "event_id", &self.lists.event_id, &[])?;
        add_new_slice_field_to(&detector, "event_time_zero", &self.lists.event_time_zero, &[("offset",""), ("units","ns")])?;
        add_new_slice_field_to(&detector, "event_time_offset", &self.lists.event_time_offset, &[])?;
        Ok(())
    }
}