mod classes;
mod logs;
#[cfg(test)]
mod mock_nexus_file;
mod nexus_file;
mod units;

use hdf5::Group;
use std::path::Path;

use crate::{
    hdf5_handlers::{ConvertResult, GroupExt, NexusHDF5Result},
    run_engine::{run_messages::HandlesAllNexusMessages, RunParameters},
    NexusSettings,
};

pub(crate) const DATETIME_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%z";

pub(crate) use classes::nexus_class;
pub(crate) use logs::{AlarmMessage, LogMessage};
#[cfg(test)]
pub(crate) use mock_nexus_file::NexusNoFile;
pub(crate) use nexus_file::NexusFile;
pub(crate) use units::{DatasetUnitExt, NexusUnits};

pub(crate) trait NexusSchematic: Sized {
    const CLASS: &str;
    type Settings;

    /// Creates a new instance of Self with new structure created in `group`
    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self>;
    /// Creates a new instance of Self with structure populated from `group`
    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self>;

    /// Creates a new hdf5 group in `parent` and calls `build_group_structure` on it,
    /// then wraps the result in `NexusGroup`
    fn build_new_group(
        parent: &Group,
        name: &str,
        settings: &Self::Settings,
    ) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent
            .add_new_group(name, Self::CLASS)
            .err_group(parent)?;
        let schematic = Self::build_group_structure(&group, settings).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }

    /// Opens the named hdf5 group in `parent` and calls `populate_group_structure` on it,
    /// then wraps the result in `NexusGroup`
    fn open_group(parent: &Group, name: &str) -> NexusHDF5Result<NexusGroup<Self>> {
        let group = parent.get_group(name).err_group(parent)?;
        let schematic = Self::populate_group_structure(&group).err_group(parent)?;
        Ok(NexusGroup { group, schematic })
    }
}

pub(crate) trait NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()>;
}

pub(crate) struct NexusGroup<S: NexusSchematic> {
    group: Group,
    schematic: S,
}

impl<S: NexusSchematic> NexusGroup<S> {
    pub(crate) fn open_from_existing_group(group: Group) -> NexusHDF5Result<Self> {
        let schematic = S::populate_group_structure(&group)?;
        Ok(Self { group, schematic })
    }
    pub(crate) fn get_name(&self) -> String {
        self.group.name()
    }

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
    fn flush(&self) -> NexusHDF5Result<()>;
    fn close(self) -> NexusHDF5Result<()>;
}
