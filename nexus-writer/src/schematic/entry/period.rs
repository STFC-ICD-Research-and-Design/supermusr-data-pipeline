use hdf5::Group;

use crate::{hdf5_handlers::NexusHDF5Result, nexus::{run_messages::InitialiseNewNexusRun, ChunkSizeSettings}, schematic::{NexusMessageHandler, NexusSchematic}};

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


impl NexusMessageHandler<InitialiseNewNexusRun<'_>> for Period {
    fn handle_message(&mut self, InitialiseNewNexusRun(_): &InitialiseNewNexusRun<'_>) -> NexusHDF5Result<()> {
        Ok(())
    }
}