#[cfg(test)]
mod mock_nexus_file;
mod nexus_file;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    run_engine::{run_messages::HandlesAllNexusMessages, ChunkSizeSettings, RunParameters},
};
#[cfg(test)]
pub(crate) use mock_nexus_file::NexusNoFile;
pub(crate) use nexus_file::NexusFile;
use std::path::Path;

pub(crate) trait NexusFileInterface: Sized + HandlesAllNexusMessages {
    fn build_new_file(file_path: &Path, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self>;
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self>;
    
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters>;

    fn flush(&self) -> NexusHDF5Result<()>;
    fn close(self) -> NexusHDF5Result<()>;
}
