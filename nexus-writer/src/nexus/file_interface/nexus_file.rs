//! This module defines the [NexusFile] struct which owns both the hdf5 [File] handle,
//! as well as the `nexus_structure::Root` object. When injected into the [NexusEngine]
//! struct, an instance of [NexusFile] is created by [NexusEngine], and all file handling
//! is controlled via this object.
//!
//! [NexusEngine]: crate::run_engine::NexusEngine
use super::NexusFileInterface;
use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{NexusMessageHandler, NexusSchematic},
    nexus_structure::Root,
    run_engine::{run_messages::HandlesAllNexusMessages, ChunkSizeSettings, RunParameters},
};
use hdf5::File;
use std::path::Path;

/// Encapsulates the creation, loading, and message handling of a NeXus file.
pub(crate) struct NexusFile {
    /// Handle to the hdf5 `File` object
    file: File,
    /// Entry point to the NeXus file, all messages are passed through here
    root: Root,
}

/// This implementation is sufficient to prove that [NexusFile] implements [NexusMessageHandler]
/// for all the required messages.
impl HandlesAllNexusMessages for NexusFile {}

impl NexusFileInterface for NexusFile {
    /// Creates a new NeXus file and initialises a new [Root] group structure with it.
    /// # Parameters
    /// - file_path: path at which to create the file.
    /// - settings: hdf5 chunk sizes to use.
    /// # Error Modes
    /// - Propagates [File::create()] errors.
    /// - Propagates [Root::build_group_structure()] errors.
    /// 
    /// [Root]: crate::nexus_structure::Root
    fn build_new_file(file_path: &Path, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self> {
        let file = File::create(file_path)?;
        let root = Root::build_group_structure(&file, settings)?;
        Ok(Self { file, root })
    }

    /// Implementation should open the NeXus file and populate a new [Root] group structure with its data.
    /// # Parameters
    /// - file_path: path of the file to open.
    /// # Error Modes
    /// - Propagates [File::open_rw()] errors.
    /// - Propagates [Root::populate_group_structure()] errors.
    /// 
    /// [Root]: crate::nexus_structure::Root
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self> {
        let file = File::open_rw(file_path)?;
        let root = Root::populate_group_structure(&file)?;
        Ok(Self { file, root })
    }

    /// Flushes the hdf5 [File] object to disk.
    /// # Error Modes
    /// Hdf5 errors are propagated.
    fn flush(&self) -> NexusHDF5Result<()> {
        Ok(self.file.flush()?)
    }

    /// Creates a [RunParameters] object from the NeXus file.
    /// # Error Modes
    /// Errors from [Root::extract_run_parameters()] are propagated.
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        self.root.extract_run_parameters()
    }

    /// Takes ownership and closes the hdf5 file.
    /// # Error Modes
    /// Hdf5 errors are propagated.
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
