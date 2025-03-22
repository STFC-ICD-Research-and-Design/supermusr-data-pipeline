use std::path::Path;

use hdf5::{File, Group};

use crate::{
    hdf5_handlers::{ConvertResult, NexusHDF5Result},
    nexus_structure::Root,
    run_engine::{
        RunParameters, NexusSettings
    }
};

use super::{NexusFileInterface, NexusMessageHandler, NexusSchematic};

pub(crate) struct NexusFile {
    file: File,
    root: Root,
}

impl NexusFileInterface for NexusFile {
    fn build_new_file(file_path: &Path, nexus_settings: &NexusSettings) -> NexusHDF5Result<Self> {
        let file = File::create(file_path)?;
        let root = Root::build_group_structure(&file, nexus_settings)?;
        Ok(Self { file, root })
    }

    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self> {
        let file = File::create(file_path)?;
        let root = Root::populate_group_structure(&file)?;
        Ok(Self { file, root })
    }

    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        self.root.extract_run_parameters()
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
