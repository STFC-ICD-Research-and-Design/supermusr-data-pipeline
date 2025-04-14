use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{NexusMessageHandler, NexusSchematic},
    nexus_structure::Root,
    run_engine::{run_messages::HandlesAllNexusMessages, ChunkSizeSettings, RunParameters},
};
use super::NexusFileInterface;
use hdf5::File;
use std::path::Path;

pub(crate) struct NexusFile {
    file: File,
    root: Root,
}

impl HandlesAllNexusMessages for NexusFile {}

impl NexusFileInterface for NexusFile {
    fn build_new_file(file_path: &Path, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self> {
        let file = File::create(file_path)?;
        let root = Root::build_group_structure(&file, settings)?;
        Ok(Self { file, root })
    }

    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self> {
        let file = File::open_rw(file_path)?;
        let root = Root::populate_group_structure(&file)?;
        Ok(Self { file, root })
    }

    fn flush(&self) -> NexusHDF5Result<()> {
        Ok(self.file.flush()?)
    }

    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        self.root.extract_run_parameters()
    }

    fn close(self) -> NexusHDF5Result<()> {
        Ok(self.file.close()?)
    }
}

impl<M> NexusMessageHandler<M> for NexusFile
where
    Root: NexusMessageHandler<M>,
{
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.root.handle_message(message)
    }
}
