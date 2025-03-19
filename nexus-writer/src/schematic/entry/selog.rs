use hdf5::Group;

use crate::NexusWriterResult;

use super::NexusSchematic;

pub(crate) struct SELog {}

impl NexusSchematic for SELog {
    const CLASS: &str = "NXselog";

    fn build_group_structure(parent: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn close_group() -> NexusWriterResult<()> {
        todo!()
    }
}
