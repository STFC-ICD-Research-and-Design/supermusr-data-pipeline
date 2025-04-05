use std::collections::{hash_map::Entry, HashMap};

use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{LogMessage, NexusGroup, NexusMessageHandler},
    nexus_structure::{log::Log, NexusSchematic},
    run_engine::{
        run_messages::{
            PushAbortRunWarning, PushIncompleteFrameWarning, PushRunLog, PushRunResumeWarning,
        },
        ChunkSizeSettings, RunLogChunkSize,
    },
};

pub(crate) struct RunLog {
    group: Group,
    runlogs: HashMap<String,NexusGroup<Log>>,
}

impl NexusSchematic for RunLog {
    const CLASS: &str = "NXrunlog";

    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
            runlogs: HashMap::default(),
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
            runlogs: group
                .groups()?
                .into_iter()
                .map(NexusGroup::<Log>::open_from_existing_group)
                .map(|group|
                    group.map(|nexus_group|(nexus_group.get_name(), nexus_group))
                )
                .collect::<Result<_, _>>()?,
        })
    }
}

impl NexusMessageHandler<PushRunLog<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushRunLog<'_>) -> NexusHDF5Result<()> {
        
        match self.runlogs.entry(message.0.get_name().to_owned()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry
                .get_mut()
                .handle_message(message.0),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(&self.group, &message.0.get_name(), &(message.0.get_type_descriptor()?, message.1.runlog))?)
                .handle_message(message.0),
        }
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
