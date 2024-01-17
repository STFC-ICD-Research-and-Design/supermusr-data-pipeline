
use std::{path::{PathBuf, Path}, collections::HashMap};

use chrono::{DateTime, Utc};
use hdf5::{file::File, H5Type, Extents, Group, SimpleExtents};
use anyhow::{anyhow, Result};
use supermusr_common::{Time, Intensity, Channel};
use supermusr_streaming_types::dev1_digitizer_event_v1_generated::DigitizerEventListMessage;


use super::{add_new_group_to, add_new_string_field_to, add_new_field_to,set_group_nx_class, add_new_slice_field_to};

#[derive(Default)]
struct EventList {
    // Indexed by event.
    event_time_offset : Vec<Time>,
    // Indexed by event.
    pulse_height : Vec<Intensity>,
    // Indexed by frame.
    event_time_zero : Vec<Time>,
    // Indexed by event.
    event_id : Vec<Channel>,
    // Indexed by frame.
    event_index: Vec<usize>,
    offset: Option<DateTime<Utc>>
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
        if let Some(offset) = self.lists.offset {
            self.lists.event_time_zero
                .push(
                    (Into::<DateTime<Utc>>::into(*data.metadata().timestamp().unwrap()) - offset)
                        .num_nanoseconds().unwrap() as Time
                );
        } else {
            self.lists.offset = Some(Into::<DateTime<Utc>>::into(*data.metadata().timestamp().unwrap())); 
            self.lists.event_time_zero.push(0);
        }
        self.lists.event_index.push(self.lists.event_id.len());
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
