use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{AlarmMessage, LogMessage, NexusClass, NexusGroup, NexusMessageHandler},
    nexus_structure::{logs::ValueLog, NexusSchematic},
    run_engine::run_messages::{PushAlarm, PushSampleEnvironmentLog},
};
use hdf5::Group;
use std::collections::{hash_map::Entry, HashMap};

pub(crate) struct SELog {
    // Helpers
    group: Group,
    // Structure
    selogs: HashMap<String, NexusGroup<ValueLog>>,
}

impl NexusSchematic for SELog {
    const CLASS: NexusClass = NexusClass::Selog;
    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
            selogs: HashMap::default(),
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
            selogs: group
                .groups()?
                .into_iter()
                .map(NexusGroup::<ValueLog>::open_from_existing_group)
                .map(|group| group.map(|nexus_group| (nexus_group.get_name(), nexus_group)))
                .collect::<Result<_, _>>()?,
        })
    }
}

impl NexusMessageHandler<PushSampleEnvironmentLog<'_>> for SELog {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(&mut self, message: &PushSampleEnvironmentLog<'_>) -> NexusHDF5Result<()> {
        match self.selogs.entry(message.get_name()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(ValueLog::build_new_group(
                    &self.group,
                    &message.get_name(),
                    &(),
                )?)
                .handle_message(message),
        }
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for SELog {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        match self.selogs.entry(message.get_name()?) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(ValueLog::build_new_group(
                    &self.group,
                    &message.get_name()?,
                    &(),
                )?)
                .handle_message(message),
        }
    }
}
