use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{NexusGroup, NexusMessageHandler},
    nexus_structure::{log::ValueLog, NexusSchematic},
    run_engine::{
        run_messages::{PushAlarm, PushSampleEnvironmentLog},
        ChunkSizeSettings,
    },
};

pub(crate) struct SELog {
    selogs: Vec<NexusGroup<ValueLog>>,
}

impl NexusSchematic for SELog {
    const CLASS: &str = "NXselog";
    type Settings = ChunkSizeSettings;

    fn build_group_structure(_: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            selogs: Vec::default(),
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            selogs: group
                .groups()?
                .into_iter()
                .map(NexusGroup::<ValueLog>::open_from_existing_group)
                .collect::<Result<_, _>>()?,
        })
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
