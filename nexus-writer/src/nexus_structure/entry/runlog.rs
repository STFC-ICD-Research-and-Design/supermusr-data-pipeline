use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{NexusGroup, NexusMessageHandler},
    nexus_structure::{log::Log, NexusSchematic},
    run_engine::{
        run_messages::{
            PushAbortRunWarning, PushIncompleteFrameWarning, PushRunLog, PushRunResumeWarning,
        },
        ChunkSizeSettings, RunLogChunkSize,
    },
};

pub(crate) struct RunLog {
    runlogs: Vec<NexusGroup<Log>>,
}

impl NexusSchematic for RunLog {
    const CLASS: &str = "NXrunlog";

    type Settings = ();

    fn build_group_structure(_: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            runlogs: Vec::default(),
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            runlogs: group
                .groups()?
                .into_iter()
                .map(NexusGroup::<Log>::open_from_existing_group)
                .collect::<Result<_, _>>()?,
        })
    }
}

impl NexusMessageHandler<PushRunLog<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushRunLog<'_>) -> NexusHDF5Result<()> {
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
