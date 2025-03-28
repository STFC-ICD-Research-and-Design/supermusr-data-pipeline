use hdf5::Group;

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus::{LogMessage, NexusGroup, NexusMessageHandler},
    nexus_structure::{log::ValueLog, NexusSchematic},
    run_engine::{
        run_messages::{PushAlarm, PushSampleEnvironmentLog},
        ChunkSizeSettings,
    },
};

pub(crate) struct SELog {
    group: Group,
    selogs: Vec<NexusGroup<ValueLog>>,
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
        let value_log = if let Some(selog) = self
            .selogs
            .iter_mut()
            .find(|selog| selog.get_name() == name)
        {
            selog
        } else {
            let group =
                ValueLog::build_new_group(&self.group, name, &message.get_value_log_settings()?)?;
            self.selogs.push(group);
            self.selogs
                .last_mut()
                .expect("Vec is non-empty, this should never happen")
        };
        value_log.handle_message(message.get_value_log_message())
    }
}

impl NexusMessageHandler<PushAlarm<'_>> for SELog {
    fn handle_message(&mut self, message: &PushAlarm<'_>) -> NexusHDF5Result<()> {
        todo!()
    }
}
