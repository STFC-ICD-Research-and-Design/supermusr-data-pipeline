mod source;

use hdf5::{types::VarLenUnicode, Dataset, Group};
use source::Source;

use crate::{
    hdf5_handlers::{DatasetExt, GroupExt, NexusHDF5Result},
    nexus_structure::{NexusGroup, NexusMessageHandler, NexusSchematic},
    run_engine::run_messages::PushRunStart,
};

mod labels {
    pub(super) const NAME: &str = "name";
    pub(super) const SOURCE: &str = "source";
}

pub(crate) struct Instrument {
    name: Dataset,
    source: NexusGroup<Source>,
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
        Ok(Self{
            name: group.get_dataset(labels::NAME)?,
            source: Source::open_group(group,labels::SOURCE)?,
        })
    }

    fn close_group() -> NexusHDF5Result<()> {
        todo!()
    }
}

impl NexusMessageHandler<PushRunStart<'_>> for Instrument {
    fn handle_message(
        &mut self,
        PushRunStart(run_start): &PushRunStart<'_>,
    ) -> NexusHDF5Result<()> {
        self.name
            .set_string_to(&run_start.instrument_name().unwrap_or_default())?;
        self.source.handle_message(&PushRunStart(*run_start))?;
        Ok(())
    }
}
