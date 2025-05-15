//! This module defines the `NexusFileInterface` trait which allows the `NexusEngine` and `Run`
//! structs to interact with the hdf5 functionality via dependency injection.
//! Both the `NexusFile` struct and the `NexusNoFile` mock struct implements this,
//! which allows `NexusEngine` and `Run` to be tested with or without actual hdf5 file interactions.
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
    /// Implementation should create a new NeXus file and initialise a new [Root] group structure with it.
    /// # Parameters
    /// - file_path: path at which to create the file.
    /// - settings: hdf5 chunk sizes to use.
    /// # Error Modes
    /// Implementation should propagate any errors.
    ///
    /// [Root]: crate::nexus_structure::Root
    fn build_new_file(file_path: &Path, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self>;

    /// Implementation should open the NeXus file and populate a new [Root] group structure with its data.
    /// # Parameters
    /// - file_path: path of the file to open.
    /// # Error Modes
    /// Implementation should propagate any errors.
    ///
    /// [Root]: crate::nexus_structure::Root
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self>;

    /// Implementation should create a [RunParameters] object from the NeXus file.
    /// # Error Modes
    /// Implementation should propagate any errors.
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters>;

    /// Implementation should flush the hdf5 [File] object to disk.
    /// # Error Modes
    /// Implementation should propagate hdf5 errors.
    ///
    /// [File]: hdf5::File
    fn flush(&self) -> NexusHDF5Result<()>;

    /// Implementation should take ownership and close the hdf5 file.
    /// # Error Modes
    /// Implementation should propagate hdf5 errors.
    fn close(self) -> NexusHDF5Result<()>;
}
