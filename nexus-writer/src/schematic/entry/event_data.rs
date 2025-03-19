use hdf5::Group;

use crate::{schematic::NexusSchematic, NexusWriterResult};

pub(crate) struct EventData {}

impl NexusSchematic for EventData {
    const CLASS: &str = "NXeventdata";

    fn build_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}
