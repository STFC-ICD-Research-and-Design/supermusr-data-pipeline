use super::entry::Entry;
use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};

use crate::{
    hdf5_handlers::{NexusGroup, NexusSchematic},
    nexus::{DatasetExt, GroupExt, HasAttributesExt, NexusWriterError},
    NexusWriterResult,
};

struct Root {
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

    fn build_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self {
            hdf5_version: group.add_attribute_to("HDF5_version", "")?,
            nexus_version: group.add_attribute_to("NeXuS_version", "")?,
            file_name: group.add_attribute_to("file_name", "")?,
            file_time: group.add_attribute_to("file_time", "")?,
            raw_data_1: Entry::build_new_group(&group, "root")?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self {
            hdf5_version: group.get_attribute("HDF5_version")?,
            nexus_version: group.get_attribute("NeXuS_version")?,
            file_name: group.get_attribute("file_name")?,
            file_time: group.get_attribute("file_time")?,
            raw_data_1: Entry::open_group(&group, "raw_data_1")?,
        })
    }

    fn close_group() -> NexusWriterResult<()> {
        Ok(())
    }
}
