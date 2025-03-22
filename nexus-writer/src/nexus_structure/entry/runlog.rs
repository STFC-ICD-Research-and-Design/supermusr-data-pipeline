use hdf5::Group;

use crate::{
    error::NexusWriterResult, hdf5_handlers::NexusHDF5Result, run_engine::ChunkSizeSettings,
    nexus_structure::NexusSchematic,
};

pub(crate) struct RunLog {}

impl NexusSchematic for RunLog {
    const CLASS: &str = "NXrunlog";
    type Settings = ChunkSizeSettings;

    fn build_group_structure(this: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}
