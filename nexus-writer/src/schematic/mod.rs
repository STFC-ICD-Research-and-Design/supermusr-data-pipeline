mod entry;
mod root;

use std::path::Path;

use hdf5::{File, Group};
use root::Root;

use crate::{
    error::NexusWriterResult,
    hdf5_handlers::{ConvertResult, NexusHDF5Result},
    nexus::{GroupExt, NexusDateTime}, NexusSettings,
};

pub(crate) trait NexusSchematic: Sized {
    const CLASS: &str;
    type Settings;

    fn build_group_structure(parent: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self>;
    fn populate_group_structure(parent: &Group) -> NexusHDF5Result<Self>;

    fn build_new_group(parent: &Group, name: &str, settings: &Self::Settings) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent
            .add_new_group_to(name, Self::CLASS)
            .err_group(parent)?;
        let schematic = Self::build_group_structure(&group, settings).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }

    fn open_group(parent: &Group, name: &str) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent.get_group(name).err_group(parent)?;
        let schematic = Self::populate_group_structure(&group).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }
    fn close_group() -> NexusHDF5Result<()>;
}

pub(crate) struct NexusGroup<S: NexusSchematic> {
    group: Group,
    schematic: S,
}

pub(crate) trait NexusFileInterface : Sized {
    fn build_new_file(file_path: &Path, nexus_settings: &NexusSettings) -> NexusHDF5Result<Self>;
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self>;
}

pub(crate) trait NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()>;
}

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
}

impl<M> NexusMessageHandler<M> for NexusFile where Root : NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.root.handle_message(message)
    }
}

pub(crate) struct NexusNoFile;

impl NexusFileInterface for NexusNoFile {
    fn build_new_file(_: &Path, _: &NexusSettings) -> NexusHDF5Result<Self> {
        Ok(Self)
    }
    
    fn open_from_file(_: &Path) -> NexusHDF5Result<Self> {
        Ok(Self)
    }
}

impl<M> NexusMessageHandler<M> for NexusNoFile where Root : NexusMessageHandler<M> {
    fn handle_message(&mut self, _: &M) -> NexusHDF5Result<()> {
        Ok(())
    }
}