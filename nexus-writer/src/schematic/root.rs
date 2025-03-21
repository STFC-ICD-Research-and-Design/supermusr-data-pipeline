use super::{entry::Entry, NexusGroup, NexusMessageHandler, NexusSchematic};
use chrono::{SecondsFormat, Utc};
use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};

use crate::{
    error::{NexusWriterError, NexusWriterResult}, hdf5_handlers::{AttributeExt, DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result}, NexusSettings
};

mod labels {
    pub(super) const HDF5_VERSION: &str = "HDF5_version";
    pub(super) const NEXUS_VERSION: &str = "NeXuS_version";
    pub(super) const FILE_NAME: &str = "file_name";
    pub(super) const FILE_TIME: &str = "file_time";
    pub(super) const RAW_DATA_1: &str = "raw_data_1";
}

pub(super) struct Root {
    hdf5_version: Attribute,
    nexus_version: Attribute,
    file_name: Attribute,
    file_time: Attribute,
    //file.add_attribute_to("HDF5_version", "1.14.3")?; // Can this be taken directly from the nix package;
    //file.add_attribute_to("NeXus_version", "")?; // Where does this come from?
    //file.add_attribute_to("file_name", &file.filename())?; //  This should be absolutized at some point
    //file.add_attribute_to("file_time", Utc::now().to_string().as_str())?; //  This should be formatted, the nanoseconds are overkill.
    raw_data_1: NexusGroup<Entry>,
}

impl NexusSchematic for Root {
    const CLASS: &str = "NX_root";
    type Settings = NexusSettings;

    fn build_group_structure(group: &Group, settings: &NexusSettings) -> NexusHDF5Result<Self> {
        Ok(Self {
            hdf5_version: group.add_attribute_to(labels::HDF5_VERSION, &format!("{0}.{1}.{2}",hdf5::HDF5_VERSION.major,hdf5::HDF5_VERSION.minor,hdf5::HDF5_VERSION.micro))?,
            nexus_version: group.add_attribute_to(labels::NEXUS_VERSION, "")?,
            file_name: group.add_attribute_to(labels::FILE_NAME, &group.filename())?,
            file_time: group.add_attribute_to(labels::FILE_TIME, Utc::now().to_rfc3339_opts(SecondsFormat::Secs,true) .as_str())?,
            raw_data_1: Entry::build_new_group(&group, labels::RAW_DATA_1, settings)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            hdf5_version: group.get_attribute(labels::HDF5_VERSION)?,
            nexus_version: group.get_attribute(labels::NEXUS_VERSION)?,
            file_name: group.get_attribute(labels::FILE_NAME)?,
            file_time: group.get_attribute(labels::FILE_TIME)?,
            raw_data_1: Entry::open_group(&group, labels::RAW_DATA_1)?,
        })
    }

    fn close_group() -> NexusHDF5Result<()> {
        Ok(())
    }
}

impl<M> NexusMessageHandler<M> for Root where Entry : NexusMessageHandler<M> {
    fn handle_message(&mut self, message: &M) -> NexusHDF5Result<()> {
        self.raw_data_1.handle_message(message)
    }
}