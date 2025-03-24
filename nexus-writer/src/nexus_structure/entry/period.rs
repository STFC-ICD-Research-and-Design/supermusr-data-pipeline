use hdf5::{Dataset, Group};

use crate::{
    hdf5_handlers::NexusHDF5Result,
    nexus_structure::{NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::InitialiseNewNexusRun, ChunkSizeSettings, GroupExt},
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
    const CLASS: &str = "NXperiod";
    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            number: group.create_scalar_dataset::<u32>(labels::NUMBER)?,
            peroid_type: group.create_scalar_dataset::<u32>(labels::PERIOD_TYPE)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<InitialiseNewNexusRun<'_>> for Period {
    fn handle_message(
        &mut self,
        InitialiseNewNexusRun(_, _): &InitialiseNewNexusRun<'_>,
    ) -> NexusHDF5Result<()> {
        Ok(())
    }
}
