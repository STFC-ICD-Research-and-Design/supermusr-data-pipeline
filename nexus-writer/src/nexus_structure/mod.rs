//! This module represents the Nexus file structure.
//! The `entry` submodule, and all its submodules, follow the group structure
//! set out in the appropriate nexus version.
//! The 'log' submodule consists of groups that appear in the `entry` module
//! as extensible vectors of groups.

mod entry;
mod logs;

use crate::{
    hdf5_handlers::{HasAttributesExt, NexusHDF5Result},
    nexus::{NexusClass, NexusGroup, NexusMessageHandler, NexusSchematic},
    run_engine::{ChunkSizeSettings, RunParameters},
};
use chrono::{SecondsFormat, Utc};
use entry::Entry;
use hdf5::{Attribute, Group};

mod labels {
    pub(super) const HDF5_VERSION: &str = "HDF5_version";
    pub(super) const NEXUS_VERSION: &str = "NeXuS_version";
    pub(super) const FILE_NAME: &str = "file_name";
    pub(super) const FILE_TIME: &str = "file_time";
    pub(super) const RAW_DATA_1: &str = "raw_data_1";
}

pub(crate) struct Root {
    _hdf5_version: Attribute,
    _nexus_version: Attribute,
    _file_name: Attribute,
    _file_time: Attribute,
    raw_data_1: NexusGroup<Entry>,
}

impl Root {
    pub(super) fn extract_run_parameters(&self) -> NexusHDF5Result<RunParameters> {
        self.raw_data_1.extract(Entry::extract_run_parameters)
    }
}

impl NexusSchematic for Root {
    const CLASS: NexusClass = NexusClass::Root;
    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, settings: &ChunkSizeSettings) -> NexusHDF5Result<Self> {
        Ok(Self {
            _hdf5_version: group.add_constant_string_attribute(
                labels::HDF5_VERSION,
                &format!(
                    "{0}.{1}.{2}",
                    hdf5::HDF5_VERSION.major,
                    hdf5::HDF5_VERSION.minor,
                    hdf5::HDF5_VERSION.micro
                ),
            )?,
            _nexus_version: group.add_constant_string_attribute(labels::NEXUS_VERSION, "")?, // Where does this come from?
            _file_name: group
                .add_constant_string_attribute(labels::FILE_NAME, &group.filename())?,
            _file_time: group.add_constant_string_attribute(
                labels::FILE_TIME,
                Utc::now()
                    .to_rfc3339_opts(SecondsFormat::Secs, true)
                    .as_str(),
            )?,
            raw_data_1: Entry::build_new_group(group, labels::RAW_DATA_1, settings)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            _hdf5_version: group.get_attribute(labels::HDF5_VERSION)?,
            _nexus_version: group.get_attribute(labels::NEXUS_VERSION)?,
            _file_name: group.get_attribute(labels::FILE_NAME)?,
            _file_time: group.get_attribute(labels::FILE_TIME)?,
            raw_data_1: Entry::open_group(group, labels::RAW_DATA_1)?,
        })
    }
}

impl<M> NexusMessageHandler<M> for Root
where
    Entry: NexusMessageHandler<M>,
{
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.raw_data_1.handle_message(message)
    }
}
