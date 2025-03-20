use hdf5::Group;

use crate::{hdf5_handlers::NexusHDF5Result, nexus::ChunkSizeSettings, schematic::NexusSchematic};

pub(crate) struct Period {}

impl NexusSchematic for Period {
    const CLASS: &str = "NXperiod";
    type Settings = ChunkSizeSettings;

    fn build_group_structure(parent: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}
