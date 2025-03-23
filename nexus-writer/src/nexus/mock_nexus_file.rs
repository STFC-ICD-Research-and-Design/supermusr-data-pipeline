use std::path::Path;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus_structure::Root,
    run_engine::{run_messages::HandlesAllNexusMessages, RunParameters},
    NexusSettings,
};

use super::{NexusFileInterface, NexusMessageHandler};

pub(crate) struct NexusNoFile;

impl HandlesAllNexusMessages for NexusNoFile {}

impl NexusFileInterface for NexusNoFile {
    fn build_new_file(_: &Path, _: &NexusSettings) -> NexusHDF5Result<Self> {
        Ok(Self)
    }

    fn open_from_file(_: &Path) -> NexusHDF5Result<Self> {
        Ok(Self)
    }

    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        unreachable!()
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
