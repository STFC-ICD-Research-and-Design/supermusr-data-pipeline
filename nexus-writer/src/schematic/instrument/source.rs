use hdf5::{types::VarLenUnicode, Dataset, Group};

use super::NexusSchematic;
use crate::{nexus::{DatasetExt, GroupExt, HasAttributesExt}, schematic::NexusGroup, NexusWriterResult};

pub(crate) struct Source {
    name: Dataset,
    source_type: Dataset,
    probe: Dataset,
}

impl NexusSchematic for Source {
    const CLASS: &str = "NXsource";

    fn build_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self {
            name: group.create_scalar_dataset::<i32>("name")?,
            source_type: group.create_scalar_dataset::<VarLenUnicode>("type")?,
            probe: group.create_scalar_dataset::<VarLenUnicode>("probe")?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}