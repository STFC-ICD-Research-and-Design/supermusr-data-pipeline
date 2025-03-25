mod classes;
#[cfg(test)]
mod mock_nexus_file;
mod nexus_file;
mod units;

use hdf5::Group;
use std::path::Path;

use crate::{
    hdf5_handlers::{ConvertResult, NexusHDF5Result},
    run_engine::{run_messages::HandlesAllNexusMessages, GroupExt, RunParameters},
    NexusSettings,
};

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

pub(crate) use classes::nexus_class;
#[cfg(test)]
pub(crate) use mock_nexus_file::NexusNoFile;
pub(crate) use nexus_file::NexusFile;
pub(crate) use units::{DatasetUnitExt, NexusUnits};

pub(crate) trait NexusSchematic: Sized {
    const CLASS: &str;
    type Settings;

    fn build_group_structure(parent: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self>;
    fn populate_group_structure(parent: &Group) -> NexusHDF5Result<Self>;

    fn build_new_group(
        parent: &Group,
        name: &str,
        settings: &Self::Settings,
    ) -> NexusHDF5Result<NexusGroup<Self>> {
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

pub(crate) trait NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()>;
}

pub(crate) struct NexusGroup<S: NexusSchematic> {
    group: Group,
    schematic: S,
}

impl<S: NexusSchematic> NexusGroup<S> {
    pub(crate) fn extract<M, F: Fn(&S) -> M>(&self, f: F) -> M {
        f(&self.schematic)
    }
}

impl<M, S> NexusMessageHandler<M> for NexusGroup<S>
where
    S: NexusSchematic + NexusMessageHandler<M>,
{
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.schematic
            .handle_message(message)
            .err_group(&self.group)
    }
}

pub(crate) trait NexusFileInterface: Sized + HandlesAllNexusMessages {
    fn build_new_file(file_path: &Path, nexus_settings: &NexusSettings) -> NexusHDF5Result<Self>;
    fn open_from_file(file_path: &Path) -> NexusHDF5Result<Self>;
    fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters>;
}
