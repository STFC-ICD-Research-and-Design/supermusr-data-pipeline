mod entry;
mod root;

use hdf5::Group;

use crate::{
    error::NexusWriterResult,
    hdf5_handlers::{ConvertResult, NexusHDF5Result},
    nexus::GroupExt,
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
