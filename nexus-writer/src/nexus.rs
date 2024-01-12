
use std::path::PathBuf;

use hdf5::{file::File, H5Type, Extents, Group, SimpleExtents};
use anyhow::{anyhow, Result};
use ndarray::s;
use supermusr_streaming_types::{dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage, dev1_digitizer_event_v1_generated::DigitizerEventListMessage, aev1_frame_assembled_event_v1_generated::FrameAssembledEventListMessage};

pub(crate) struct Nexus {
    file : Box<File>,
}

impl Nexus {
    pub(crate) fn create_file (filename : &PathBuf) -> Result<Self> {
        let file = File::create(filename)?;

        //  NXroot
        file.new_dataset_builder().with_data(&["My File Name".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/file_name")?;
        file.new_dataset_builder().with_data(&["Now".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/file_time")?;
        
        //  NXroot/NXentry
        file.new_dataset_builder().with_data(&[2]).create("/NXroot/NXentry/IDF_version")?;
        file.new_dataset_builder().with_data(&["".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/NXentry/definition")?;
        file.new_dataset_builder().with_data(&[2]).create("/NXroot/NXentry/run_number")?;
        file.new_dataset_builder().with_data(&["".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/NXentry/title")?;
        file.new_dataset_builder().with_data(&["".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/NXentry/start_time")?;
        file.new_dataset_builder().with_data(&["".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/NXentry/end_time")?;
        file.new_dataset_builder().with_data(&["".parse::<hdf5::types::VarLenUnicode>().unwrap()]).create("/NXroot/NXentry/experiment_identifier")?;

        //  NXroot/NXentry/NXinstrument
        //  NXroot/NXentry/NXdata

        Ok(Nexus { file: Box::new(file) })
    }
    pub(crate) fn push_trace (&mut self, data : &DigitizerAnalogTraceMessage) -> Result<()> {
        self.file.new_dataset_builder().with_data(&[0]/*data.channels().unwrap().get(0).voltage().unwrap()*/).create("/NXroot/NXentry/detector_1/counts")?;
        Ok(())
    }
    pub(crate) fn push_event (&mut self, data : &FrameAssembledEventListMessage) -> Result<()> {
        Ok(())
    }
    pub(crate) fn close (self) -> Result<()> {
        Ok(self.file.close()?)
    }
}