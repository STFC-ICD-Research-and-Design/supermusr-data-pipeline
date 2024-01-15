
use std::{path::PathBuf, fmt::Display};

use hdf5::{file::File, H5Type, Extents, Group, SimpleExtents};
use anyhow::{anyhow, Result};
use ndarray::s;
use supermusr_streaming_types::{dat1_digitizer_analog_trace_v1_generated::{DigitizerAnalogTraceMessage, ChannelTrace}, dev1_digitizer_event_v1_generated::DigitizerEventListMessage, aev1_frame_assembled_event_v1_generated::FrameAssembledEventListMessage};

#[derive(Default)]
pub(crate) struct Nexus {
    file : Option<File>,
    cur_path : String,
    num_traces: usize,
    run_number: usize,
}

impl Nexus {
    pub(crate) fn new () -> Self {
        Self::default()
    }
    pub(crate) fn is_running (&self) -> bool {
        self.file.is_some()
    }

    //  Field and Path Methods
    pub(crate) fn set_cur_path (&mut self, new_path: &str) {
        self.cur_path = new_path.to_owned();
    }
    pub(crate) fn add_string_field (&mut self, name: &str, content: &str) -> Result<()> {
        if let Some(file) = &self.file {
            let field_path = format!("{0}{name}",self.cur_path);
            match file.new_dataset_builder().with_data(&[content.parse::<hdf5::types::VarLenUnicode>()?]).create(field_path.as_str()) {
                Ok(data) => Ok(()),
                Err(e) => Err(anyhow!("Could not add string: {content} to {field_path}. Error {e}"))
            }
        } else {
            Ok(())
        }
    }
    pub(crate) fn add_field<T : H5Type + Display + Copy> (&mut self, name: &str, content: T) -> Result<()> {
        if let Some(file) = &self.file {
            let field_path = format!("{0}{name}",self.cur_path);
            match file.new_dataset_builder().with_data(&[content]).create(field_path.as_str()) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Could not add field: {content} to {field_path}. Error {e}"))
            }
        } else {
            Ok(())
        }
    }
    pub(crate) fn add_slice_field<T : H5Type> (&mut self, name: &str, content: &[T]) -> Result<()> {
        if let Some(file) = &self.file {
            let field_path = format!("{0}{name}",self.cur_path);
            match file.new_dataset_builder().with_data(content).create(field_path.as_str()) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Could not add slice to {field_path}. Error {e}"))
            }
        } else {
            Ok(())
        }
    }
    fn add_nx_class(&self, class: &str) -> Result<()> {
        self.add_attribute("NX_class", class)
    }

    fn add_attribute(&self, attr: &str, value: &str) -> Result<()> {
        if let Some(file) = &self.file {
            let field_path = format!("{0}{attr}",self.cur_path);
            file.new_attr_builder().with_data(value).create(field_path.as_str())?;
        }
        Ok(())
    }

    pub(crate) fn create_file (&mut self, filename : &PathBuf) -> Result<()> {
        self.file = Some(File::create(filename)?);
        
        //  NXroot
        self.set_cur_path("/NXroot/");
        self.add_string_field("file_name", "My File Name")?;
        self.add_string_field("file_time", "Now")?;

        self.begin_entry()?;

        Ok(())
    }

    // Header and 
    fn begin_entry(&mut self) -> Result<()> {
        if self.file.is_some() {
            //  NXroot/NXentry
            self.set_cur_path("/NXroot/NXentry/");
            self.add_nx_class("NXentry")?;

            self.add_field("IDF_version", 2)?;
            self.add_string_field("definition", "muonTD")?;
            self.add_field("run_number", self.run_number)?;
            self.add_string_field("experiment_identifier", "")?;
            self.add_string_field("start_time", "")?;

            self.set_cur_path(&format!("/NXroot/NXentry/detector_1"));
            self.add_nx_class("NXdetector")?;
        }

        Ok(())
    }
    
    fn add_metadata_group (&mut self, data : &DigitizerAnalogTraceMessage) -> Result<()> {
        self.set_cur_path(&format!("/NXroot/NXentry/detector_1/traces/trace_{0}/metadata/", self.num_traces));
        self.add_field("frame_number", data.metadata().frame_number())?;
        self.add_field("period_number", data.metadata().period_number())?;
        self.add_field("protons_per_pulse", data.metadata().protons_per_pulse())?;
        self.add_field("running", data.metadata().running())?;
        self.add_field("veto_flags", data.metadata().veto_flags())?;
        
        self.set_cur_path(&format!("/NXroot/NXentry/detector_1/traces/trace_{0}/metadata/timestamp/", self.num_traces));
        match data.metadata().timestamp() {
            Some(_) => {
                self.add_string_field("value", "Now")?;
            },
            None => ()
        }
        Ok(())
    }
    
    fn add_channel_group (&mut self, index : usize, channel : &ChannelTrace) -> Result<()> {
        self.set_cur_path(&format!("/NXroot/NXentry/detector_1/traces/trace_{0}/channel_{1}/", self.num_traces, index));
        self.add_field("channel_id", channel.channel())?;

        let voltage = channel.voltage().ok_or(anyhow!("No voltage data found in DAT message channel {0}.",channel.channel()))?;
        self.add_field("num_samples", voltage.len())?;
        self.add_slice_field("voltage", &voltage.into_iter().collect::<Vec<_>>())?;
        Ok(())
    }
    pub(crate) fn add_trace_group (&mut self, data : &DigitizerAnalogTraceMessage) -> Result<()> {
        if self.file.is_some() {
            //self.set_cur_path("/NXroot/NXentry/runlog/");
            self.set_cur_path(&format!("/NXroot/NXentry/detector_1/traces/trace_{0}/", self.num_traces));
            
            self.add_field("digitizer_id", data.digitizer_id())?;
            let channels = data.channels().ok_or(anyhow!("No channel data found in DAT message."))?;
            self.add_field("num_channels", channels.len())?;

            self.add_metadata_group(data)?;

            for (c,ch) in channels.iter().enumerate() {
                self.add_channel_group(c,&ch)?
            }
            self.num_traces += 1;
        }
        Ok(())
    }

    pub(crate) fn add_event_group (&mut self, data : &FrameAssembledEventListMessage) -> Result<()> {
        Ok(())
    }
    
    fn end_entry(&mut self) -> Result<()> {
        self.set_cur_path("/NXroot/NXentry/");
        if self.file.is_some() {
            self.add_string_field("end_time", "")?;
            self.add_field("num_traces", self.num_traces)?;
        }

        Ok(())
    }
    pub(crate) fn close_file (&mut self) -> Result<()> {
        self.end_entry()?;

        if let Some(file) = self.file.take() {
            file.close()?
        }
        Ok(())
    }
}