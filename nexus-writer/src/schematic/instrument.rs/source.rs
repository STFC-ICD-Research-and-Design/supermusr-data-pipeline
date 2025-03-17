use hdf5::Dataset;

use super::NexusSchematic;
use crate::{NexusWriterResult, nexus::{HasAttributesExt, DatasetExt, GroupExt}};

pub(crate) struct Source {
    source_name: Dataset,
    source_type: Dataset,
    source_probe: Dataset,
}

impl NexusSchematic for Source {
    const CLASS: &str = "NXsource";

    fn build_new_group(parent: &hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn open_group(parent: &hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}