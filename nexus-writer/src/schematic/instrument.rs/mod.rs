mod source;

use hdf5::Dataset;

use super::NexusSchematic;
use crate::{NexusWriterResult, nexus::{HasAttributesExt, DatasetExt, GroupExt}};

pub(crate) struct Instrument {
    name: Dataset,
    source: Source
}

impl NexusSchematic for Instrument {
    const CLASS: &str = "NXinstrument";

    fn build_new_group(parent: &hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
        .create_constant_string_dataset("instrument_name", "")?
    }

    fn open_group(parent: &hdf5::Group) -> crate::NexusWriterResult<super::NexusGroup<Self>> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}