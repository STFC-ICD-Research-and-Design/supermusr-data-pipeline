use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result, nexus::NexusMessageHandler, nexus_structure::NexusSchematic, run_engine::{run_messages::{PushAbortRunWarning, PushIncompleteFrameWarning, PushRunLogData, PushRunResumeWarning}, ChunkSizeSettings}
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



impl NexusMessageHandler<PushRunLogData<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushRunLogData<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}


impl NexusMessageHandler<PushRunResumeWarning<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushRunResumeWarning<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<PushIncompleteFrameWarning<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushIncompleteFrameWarning<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<PushAbortRunWarning<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushAbortRunWarning<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}