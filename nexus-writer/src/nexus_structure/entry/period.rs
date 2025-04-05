use hdf5::{Dataset, Group};

use crate::{
    hdf5_handlers::{GroupExt, NexusHDF5Result}, nexus::nexus_class, nexus_structure::{NexusMessageHandler, NexusSchematic}, run_engine::{run_messages::PushFrameEventList, ChunkSizeSettings}
};

mod labels {
    pub(super) const NUMBER: &str = "number";
    pub(super) const PERIOD_TYPE: &str = "type";
}

pub(crate) struct Period {
    number: Dataset,
    peroid_type: Dataset,
}

impl NexusSchematic for Period {
    const CLASS: &str = nexus_class::PERIOD;
    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            number: group.create_scalar_dataset::<u32>(labels::NUMBER)?,
            peroid_type: group.create_scalar_dataset::<u32>(labels::PERIOD_TYPE)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            number: group.get_dataset(labels::NUMBER)?,
            peroid_type: group.get_dataset(labels::PERIOD_TYPE)?,
        })
    }
}

impl NexusMessageHandler<PushFrameEventList<'_>> for Period {
    fn handle_message(
        &mut self,
        PushFrameEventList(message): &PushFrameEventList<'_>,
    ) -> NexusHDF5Result<()> {
        Ok(())
    }
}
