use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{AlarmMessage, LogMessage, NexusGroup, NexusMessageHandler},
    nexus_structure::{log::ValueLog, NexusSchematic},
    run_engine::{
        run_messages::{PushAlarm, PushSampleEnvironmentLog}, AlarmChunkSize, ChunkSizeSettings, SELogChunkSize
    },
};

pub(crate) struct SELog {
    // Helpers
    group: Group,
    // Structure
    selogs: Vec<NexusGroup<ValueLog>>,
}

impl SELog {
    fn find_value_log(&mut self, name: &str) -> Option<&mut NexusGroup<ValueLog>> {
        self.selogs
            .iter_mut()
            .find(|selog| selog.get_name() == name)
    }
    
    fn create_new_value_log(&mut self, name: &str) -> NexusHDF5Result<&mut NexusGroup<ValueLog>> {
        let group =
            ValueLog::build_new_group(&self.group, name, &())?;

        self.selogs.push(group);
        
        Ok(self.selogs
            .last_mut()
            .expect("Vec is non-empty, this should never happen"))
    }

    fn get_value_log_or_create_new(&mut self, name: &str) -> NexusHDF5Result<&mut NexusGroup<ValueLog>> {
        if let Some(selog) = self.find_value_log(name) {
            Ok(selog)
        } else {
            self.create_new_value_log(name)
        }
    }
}

impl NexusSchematic for SELog {
    const CLASS: &str = "NXselog";

    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
            selogs: Vec::default(),
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            group: group.clone(),
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
        let name = message.get_selog().get_name();
        let value_log = self.get_value_log_or_create_new(name);
        value_log.handle_message(message.get_value_log_message())
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for SELog {
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        let name = message.0.get_name()?;
        let value_log = self.get_value_log_or_create_new(name);
        value_log.handle_message(message.get_value_log_message())
    }
}
