
use hdf5::{file::File, H5Type, Extents, SimpleExtents};
use anyhow::{anyhow, Result};
use ndarray::s;
use supermusr_streaming_types::{dat1_digitizer_analog_trace_v1_generated::DigitizerAnalogTraceMessage, dev1_digitizer_event_v1_generated::DigitizerEventListMessage, aev1_frame_assembled_event_v1_generated::FrameAssembledEventListMessage};

pub(crate) struct Nexus {

}

impl Nexus {
    pub(crate) fn create_file () -> Result<Self> {
        Ok(Nexus {})
    }
    pub(crate) fn push_trace (&mut self, data : &DigitizerAnalogTraceMessage) -> Result<()> {
        Ok(())
    }
    pub(crate) fn push_event (&mut self, data : &FrameAssembledEventListMessage) -> Result<()> {
        Ok(())
    }
    pub(crate) fn write (&mut self, filename : &str) -> Result<()> {
        let file = File::create(filename)?;
        let root = file.create_group("NXroot")?;
        root.new_dataset::<i32>().shape(Extents::Scalar).create("/raw_data_1/IDF_version")?.write_scalar(&2)?;
        root.new_dataset::<u8>().shape((5,)).create("/raw_data_1/Test")?.write_slice(&['3' as u8,'f' as u8, 'r' as u8, 'g' as u8, 'u' as u8], s![0..4])?;
        file.close()?;
        Ok(())
    }
}