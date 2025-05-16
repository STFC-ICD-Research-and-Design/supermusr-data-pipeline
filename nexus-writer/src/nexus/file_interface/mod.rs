//! Defines the [NexusFileInterface] trait which allows the [NexusEngine] and [Run]
//! structs to interact with the hdf5 functionality via dependency injection.
//!
//! Both the [NexusFile] struct and the [NexusNoFile] mock struct implements this,
//! which allows `NexusEngine` and `Run` to be tested with or without actual hdf5 file interactions.
//! 
//! [NexusEngine]: crate::run_engine::NexusEngine
//! [Run]: crate::run_engine::Run
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

/// Implementation should allow the user to build a [Root] object
///
/// [Root]: crate::nexus_structure::Root
pub(crate) trait NexusFileInterface: Sized + HandlesAllNexusMessages {
    /// Creates a new NeXus file and initialise a new [Root] group structure with it.
    /// # Parameters
    /// - file_path: path at which to create the file.
    /// - settings: hdf5 chunk sizes to use.
    ///
    /// [Root]: crate::nexus_structure::Root
    fn build_new_file(file_path: &Path, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self>;

    /// Opens the NeXus file and populate a new [Root] group structure with its data.
    /// # Parameters
    /// - file_path: path of the file to open.
    ///
    /// [Root]: crate::nexus_structure::Root
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self>;

    /// Creates a [RunParameters] object from the NeXus file.
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters>;

    /// Flushes the hdf5 [File] object to disk.
    ///
    /// [File]: hdf5::File
    fn flush(&self) -> NexusHDF5Result<()>;

    /// Takes ownership and close the hdf5 file.
    fn close(self) -> NexusHDF5Result<()>;
}
