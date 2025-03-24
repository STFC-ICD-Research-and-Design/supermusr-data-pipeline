use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::NexusMessageHandler,
    nexus_structure::NexusSchematic,
    run_engine::{
        run_messages::{PushAlarm, PushSampleEnvironmentLog},
        ChunkSizeSettings,
    },
};

pub(crate) struct SELog {}

impl NexusSchematic for SELog {
    const CLASS: &str = "NXselog";
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

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for SELog {
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for SELog {
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}
