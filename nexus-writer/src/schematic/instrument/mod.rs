mod source;

use hdf5::{types::VarLenUnicode, Dataset, Group};
use source::Source;

use super::{NexusGroup, NexusSchematic};
use crate::{NexusWriterResult, nexus::{HasAttributesExt, DatasetExt, GroupExt}};

pub(crate) struct Instrument {
    name: Dataset,
    source: NexusGroup<Source>
}

impl NexusSchematic for Instrument {
    const CLASS: &str = "NXinstrument";

    fn build_group_structure(group: &Group) -> NexusWriterResult<Self> {
        Ok(Self {
            name: group.create_scalar_dataset::<VarLenUnicode>("name")?,
            source: Source::build_new_group(group, "source")?
        })
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}