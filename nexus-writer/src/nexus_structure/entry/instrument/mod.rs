mod source;

use hdf5::{types::VarLenUnicode, Dataset, Group};
use source::Source;

use crate::{
    hdf5_handlers::{DatasetExt, GroupExt, NexusHDF5Result},
    run_engine::run_messages::InitialiseNewNexusRun,
    nexus_structure::{NexusGroup, NexusMessageHandler, NexusSchematic},
};

pub(crate) struct Instrument {
    name: Dataset,
    source: NexusGroup<Source>,
}

impl Instrument {
    pub(super) fn get_instrument_name(&self) -> NexusHDF5Result<String> {
        self.name.get_string_from()
    }
}

impl NexusSchematic for Instrument {
    const CLASS: &str = "NXinstrument";
    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.create_scalar_dataset::<VarLenUnicode>("name")?,
            source: Source::build_new_group(group, "source", &())?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        todo!()
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<InitialiseNewNexusRun<'_>> for Instrument {
    fn handle_message(
        &mut self,
        InitialiseNewNexusRun(parameters): &InitialiseNewNexusRun<'_>,
    ) -> NexusHDF5Result<()> {
        self.name.set_string_to(&parameters.instrument_name)?;
        self.source
            .handle_message(&InitialiseNewNexusRun(parameters))?;
        Ok(())
    }
}
