use hdf5::Group;

use crate::NexusWriterResult;

use super::NexusSchematic;

pub(crate) struct RunLog {}

impl NexusSchematic for RunLog {
    const CLASS: &str = "NXrunlog";

    fn build_group_structure(this: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn populate_group_structure(group: &Group) -> NexusWriterResult<Self> {
        todo!()
    }

    fn close_group() -> crate::NexusWriterResult<()> {
        todo!()
    }
}
