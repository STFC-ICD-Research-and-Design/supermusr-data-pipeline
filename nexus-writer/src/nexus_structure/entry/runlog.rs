use std::collections::{hash_map::Entry, HashMap};

use hdf5::{types::TypeDescriptor, Group};

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{nexus_class, LogMessage, NexusGroup, NexusMessageHandler},
    nexus_structure::{log::Log, NexusSchematic},
    run_engine::run_messages::{
        PushAbortRunWarning, PushIncompleteFrameWarning, PushRunLog, PushRunResumeWarning,
    },
};

pub(crate) struct RunLog {
    group: Group,
    runlogs: HashMap<String, NexusGroup<Log>>,
}

impl NexusSchematic for RunLog {
    const CLASS: &str = nexus_class::RUNLOG;

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
                .map(|group| group.map(|nexus_group| (nexus_group.get_name(), nexus_group)))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl NexusMessageHandler<PushRunLog<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushRunLog<'_>) -> NexusHDF5Result<()> {
        match self.runlogs.entry(message.runlog.get_name().to_owned()) {
            Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().handle_message(message.runlog)
            }
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    &message.runlog.get_name(),
                    &(
                        message.runlog.get_type_descriptor()?,
                        message.settings.runlog,
                    ),
                )?)
                .handle_message(message.runlog),
        }
    }
}

impl NexusMessageHandler<PushRunResumeWarning<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushRunResumeWarning<'_>) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_RunResumed";
        const TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::Float(hdf5::types::FloatSize::U4);
        match self.runlogs.entry(LOG_NAME.to_string()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    LOG_NAME,
                    &(TYPE_DESCRIPTOR, message.settings.runlog),
                )?)
                .handle_message(message),
        }
    }
}

impl NexusMessageHandler<PushIncompleteFrameWarning<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushIncompleteFrameWarning<'_>) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_DigitisersPresentInIncompleteFrame";
        const TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::VarLenUnicode;
        match self.runlogs.entry(LOG_NAME.to_string()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    LOG_NAME,
                    &(TYPE_DESCRIPTOR, message.settings.runlog),
                )?)
                .handle_message(message),
        }
    }
}

impl NexusMessageHandler<PushAbortRunWarning<'_>> for RunLog {
    fn handle_message(&mut self, message: &PushAbortRunWarning<'_>) -> NexusHDF5Result<()> {
        const LOG_NAME: &str = "SuperMuSRDataPipeline_RunAborted";
        const TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::VarLenUnicode;
        match self.runlogs.entry(LOG_NAME.to_string()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    LOG_NAME,
                    &(TYPE_DESCRIPTOR, message.settings.runlog),
                )?)
                .handle_message(message),
        }
    }
}
