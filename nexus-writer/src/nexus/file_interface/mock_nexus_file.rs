use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::NexusMessageHandler,
    nexus_structure::Root,
    run_engine::{run_messages::HandlesAllNexusMessages, ChunkSizeSettings, RunParameters},
};
use super::NexusFileInterface;
use std::path::Path;

pub(crate) struct NexusNoFile;

impl HandlesAllNexusMessages for NexusNoFile {}

impl NexusFileInterface for NexusNoFile {
    fn build_new_file(_: &Path, _: &ChunkSizeSettings) -> NexusHDF5Result<Self> {
        Ok(Self)
    }

    fn open_from_file(_: &Path) -> NexusHDF5Result<Self> {
        Ok(Self)
    }

    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        unreachable!()
    }

    fn flush(&self) -> NexusHDF5Result<()> {
        Ok(())
    }

    fn close(self) -> NexusHDF5Result<()> {
        Ok(())
    }
}

impl<M> NexusMessageHandler<M> for NexusNoFile
where
    Root: NexusMessageHandler<M>,
{
    fn handle_message(&mut self, _: &M) -> NexusHDF5Result<()> {
        Ok(())
    }
}
