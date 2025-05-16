//! This module implements the `NexusNoFile` struct which mocks the `NexusFile` struct.
//! This is used for testing purposes only.
use super::NexusFileInterface;
use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::NexusMessageHandler,
    nexus_structure::Root,
    run_engine::{run_messages::HandlesAllNexusMessages, ChunkSizeSettings, RunParameters},
};
use std::path::Path;

/// Empty struct.
pub(crate) struct NexusNoFile;

impl HandlesAllNexusMessages for NexusNoFile {}

impl NexusFileInterface for NexusNoFile {
    /// Does nothing.
    fn build_new_file(_: &Path, _: &ChunkSizeSettings) -> NexusHDF5Result<Self> {
        Ok(Self)
    }

    /// Does nothing.
    fn open_from_file(_: &Path) -> NexusHDF5Result<Self> {
        Ok(Self)
    }

    /// This should never be called, panics if it is.
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        unreachable!()
    }

    /// Does nothing.
    fn flush(&self) -> NexusHDF5Result<()> {
        Ok(())
    }

    /// Does nothing.
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
