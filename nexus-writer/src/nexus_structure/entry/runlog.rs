use std::collections::{hash_map::Entry, HashMap};

use hdf5::{types::TypeDescriptor, Group};

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{nexus_class, LogMessage, NexusGroup, NexusMessageHandler},
    nexus_structure::{log::Log, NexusSchematic},
    run_engine::run_messages::{
        InternallyGeneratedLog, PushInternallyGeneratedLogWarning, PushRunLog,
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
        match self.runlogs.entry(message.runlog.get_name()) {
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

const RUN_RESUMED_LOG_NAME: &str = "SuperMuSRDataPipeline_RunResumed";
const RUN_RESUMED_TYPE_DESCRIPTOR: TypeDescriptor =
    TypeDescriptor::Float(hdf5::types::FloatSize::U4);
const INCOMPLETE_FRAME_LOG_NAME: &str = "SuperMuSRDataPipeline_DigitisersPresentInIncompleteFrame";
const INCOMPLETE_FRAME_TYPE_DESCRIPTOR: TypeDescriptor =
    TypeDescriptor::Float(hdf5::types::FloatSize::U4);
const RUN_ABORTED_LOG_NAME: &str = "SuperMuSRDataPipeline_RunAborted";
const RUN_ABORTED_TYPE_DESCRIPTOR: TypeDescriptor = TypeDescriptor::VarLenUnicode;

impl NexusMessageHandler<PushInternallyGeneratedLogWarning<'_>> for RunLog {
    fn handle_message(
        &mut self,
        message: &PushInternallyGeneratedLogWarning<'_>,
    ) -> NexusHDF5Result<()> {
        let (log_name, type_descriptor) = match message.message {
            InternallyGeneratedLog::RunResume { resume_time: _ } => {
                (RUN_RESUMED_LOG_NAME, RUN_RESUMED_TYPE_DESCRIPTOR)
            }
            InternallyGeneratedLog::IncompleteFrame { frame: _ } => {
                (INCOMPLETE_FRAME_LOG_NAME, INCOMPLETE_FRAME_TYPE_DESCRIPTOR)
            }
            InternallyGeneratedLog::AbortRun { stop_time_ms: _ } => {
                (RUN_ABORTED_LOG_NAME, RUN_ABORTED_TYPE_DESCRIPTOR)
            }
        };

        match self.runlogs.entry(log_name.to_string()) {
            Entry::Occupied(mut occupied_entry) => occupied_entry.get_mut().handle_message(message),
            Entry::Vacant(vacant_entry) => vacant_entry
                .insert(Log::build_new_group(
                    &self.group,
                    log_name,
                    &(type_descriptor, message.settings.runlog),
                )?)
                .handle_message(message),
        }
    }
}
