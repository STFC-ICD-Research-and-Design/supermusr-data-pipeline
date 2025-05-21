//! Defines group structure which contains the run logs of the run.
use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{LogMessage, NexusClass, NexusGroup, NexusMessageHandler},
    nexus_structure::{
        NexusSchematic,
        logs::{Log, LogSettings},
    },
    run_engine::run_messages::{
        InternallyGeneratedLog, PushInternallyGeneratedLogWarning, PushRunLog,
    },
};
use hdf5::{
    Group,
    types::{FloatSize, TypeDescriptor},
};
use std::collections::{HashMap, hash_map::Entry};

/// Group structure for the RunLog group.
/// Unlike most other group structures, this contains
/// a [HashMap] of [Log]-structured subgroups, indexed by strings.
pub(crate) struct RunLog {
    group: Group,
    runlogs: HashMap<String, NexusGroup<Log>>,
}

impl NexusSchematic for RunLog {
    const CLASS: NexusClass = NexusClass::Runlog;
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

/// If the run log already exists then add the data to the appropriate log,
/// otherwise create a new log and append the data to it.
impl NexusMessageHandler<PushRunLog<'_>> for RunLog {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(&mut self, message: &PushRunLog<'_>) -> NexusHDF5Result<()> {
        match self.runlogs.entry(message.get_name()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    &message.get_name(),
                    &LogSettings {
                        type_descriptor: message.get_type_descriptor()?,
                        chunk_size: message.settings.runlog,
                    },
                )?)
                .handle_message(message),
        }
    }
}

const RUN_RESUMED_LOG_NAME: &str = "SuperMuSRDataPipeline_RunResumed";
const RUN_RESUMED_TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::Float(FloatSize::U4);
const INCOMPLETE_FRAME_LOG_NAME: &str = "SuperMuSRDataPipeline_DigitisersPresentInIncompleteFrame";
const INCOMPLETE_FRAME_TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::VarLenUnicode;
const RUN_ABORTED_LOG_NAME: &str = "SuperMuSRDataPipeline_RunAborted";
const RUN_ABORTED_TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::Float(FloatSize::U4);

/// If the run log for the internally generated message already exists,
/// then add the data to the appropriate log, otherwise create a new log
/// and append the data to it.
impl NexusMessageHandler<PushInternallyGeneratedLogWarning<'_>> for RunLog {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn handle_message(
        &mut self,
        message: &PushInternallyGeneratedLogWarning<'_>,
    ) -> NexusHDF5Result<()> {
        let (log_name, type_descriptor) = match message.message {
            InternallyGeneratedLog::RunResume { .. } => {
                (RUN_RESUMED_LOG_NAME, RUN_RESUMED_TYPE_DESCRIPTOR)
            }
            InternallyGeneratedLog::IncompleteFrame { .. } => {
                (INCOMPLETE_FRAME_LOG_NAME, INCOMPLETE_FRAME_TYPE_DESCRIPTOR)
            }
            InternallyGeneratedLog::AbortRun { .. } => {
                (RUN_ABORTED_LOG_NAME, RUN_ABORTED_TYPE_DESCRIPTOR)
            }
        };

        match self.runlogs.entry(log_name.to_string()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    log_name,
                    &LogSettings {
                        type_descriptor,
                        chunk_size: message.settings.runlog,
                    },
                )?)
                .handle_message(message),
        }
    }
}
