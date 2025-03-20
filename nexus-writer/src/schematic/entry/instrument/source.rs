use hdf5::{types::VarLenUnicode, Dataset, Group};

use super::NexusSchematic;
use crate::{
    error::NexusWriterResult,
    hdf5_handlers::{DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result},
    schematic::NexusGroup,
};

pub(crate) struct Source {
    name: Dataset,
    source_type: Dataset,
    probe: Dataset,
}

impl NexusSchematic for Source {
    const CLASS: &str = "NXsource";
    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.create_scalar_dataset::<i32>("name")?,
            source_type: group.create_scalar_dataset::<VarLenUnicode>("type")?,
            probe: group.create_scalar_dataset::<VarLenUnicode>("probe")?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}
