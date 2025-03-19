mod entry;
mod root;

use entry::Entry;
use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};

use crate::{
    nexus::{DatasetExt, GroupExt, HasAttributesExt, NexusWriterError},
    NexusWriterResult,
};

pub(crate) trait NexusSchematic: Sized {
    const CLASS: &str;

    fn build_group_structure(parent: &Group) -> NexusWriterResult<Self>;
    fn populate_group_structure(parent: &Group) -> NexusWriterResult<Self>;

    fn create_and_setup_group(parent: &Group, name: &str) -> NexusWriterResult<Group> {
        Ok(parent.add_new_group_to(name, Self::CLASS)?)
    }

    fn build_new_group(parent: &Group, name: &str) -> NexusWriterResult<NexusGroup<Self>> {
        let group = Self::create_and_setup_group(parent, name)?;
        let schematic = Self::build_group_structure(&group)?;
        Ok(NexusGroup { group, schematic })
    }

    fn open_group(parent: &Group, name: &str) -> NexusWriterResult<NexusGroup<Self>> {
        let group = parent.get_group(name)?;
        let schematic = Self::populate_group_structure(&group)?;
        Ok(NexusGroup { group, schematic })
    }
    fn close_group() -> NexusWriterResult<()>;
}

pub(crate) struct NexusGroup<S: NexusSchematic> {
    group: Group,
    schematic: S,
}
