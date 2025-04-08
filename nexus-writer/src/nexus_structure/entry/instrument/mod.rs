mod source;

use hdf5::{Dataset, Group};
use source::Source;

use crate::{
    error::FlatBufferMissingError,
    hdf5_handlers::{DatasetExt, GroupExt, NexusHDF5Error, NexusHDF5Result},
    nexus::NexusClass,
    nexus_structure::{NexusGroup, NexusMessageHandler, NexusSchematic},
    run_engine::run_messages::PushRunStart,
};

/// Names of datasets/attribute and subgroups in the Entry struct
mod labels {
    pub(super) const NAME: &str = "name";
    pub(super) const SOURCE: &str = "source";
}

pub(crate) struct Instrument {
    name: Dataset,
    _source: NexusGroup<Source>,
}

impl NexusSchematic for Instrument {
    const CLASS: NexusClass = NexusClass::Instrument;
    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.create_string_dataset("name")?,
            _source: Source::build_new_group(group, "source", &())?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.get_dataset(labels::NAME)?,
            _source: Source::open_group(group, labels::SOURCE)?,
        })
    }
}

impl NexusMessageHandler<PushRunStart<'_>> for Instrument {
    fn handle_message(
        &mut self,
        PushRunStart(run_start): &PushRunStart<'_>,
    ) -> NexusHDF5Result<()> {
        self.name
            .set_string(run_start.instrument_name().ok_or_else(|| {
                NexusHDF5Error::new_flatbuffer_missing(FlatBufferMissingError::InstrumentName)
            })?)
    }
}
