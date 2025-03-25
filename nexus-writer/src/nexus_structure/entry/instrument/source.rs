use hdf5::{types::VarLenUnicode, Dataset, Group};

use super::NexusSchematic;
use crate::{
    error::NexusWriterResult,
    hdf5_handlers::{DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result},
    nexus_structure::{NexusGroup, NexusMessageHandler},
    run_engine::run_messages::{InitialiseNewNexusRun, PushRunStart},
};

mod labels {
    pub(super) const NAME: &str = "name";
    pub(super) const SOURCE_TYPE: &str = "type";
    pub(super) const PROBE: &str = "probe";
}

pub(crate) struct Source {
    name: Dataset,
    source_type: Dataset,
    probe: Dataset,
}

impl NexusSchematic for Source {
    const CLASS: &str = "NXsource";
    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.create_scalar_dataset::<i32>(labels::NAME)?,
            source_type: group.create_scalar_dataset::<VarLenUnicode>(labels::SOURCE_TYPE)?,
            probe: group.create_scalar_dataset::<VarLenUnicode>(labels::PROBE)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.get_dataset(labels::NAME)?,
            source_type: group.get_dataset(labels::SOURCE_TYPE)?,
            probe: group.get_dataset(labels::PROBE)?,
        })
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<PushRunStart<'_>> for Source {
    fn handle_message(
        &mut self,
        PushRunStart(run_start): &PushRunStart<'_>,
    ) -> NexusHDF5Result<()> {
        self.name
            .set_string_to(&run_start.instrument_name().unwrap_or_default())?;
        self.source_type.set_string_to("")?;
        self.probe.set_string_to("")?;
        Ok(())
    }
}
