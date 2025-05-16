//! Defines [Instrument] group structure which contains details about the instrument used to probe the sample.
//! Currently unknown where this data is obtained from.
mod source;

use crate::{
    error::FlatBufferMissingError,
    hdf5_handlers::{DatasetExt, GroupExt, NexusHDF5Result},
    nexus::NexusClass,
    nexus_structure::{NexusGroup, NexusMessageHandler, NexusSchematic},
    run_engine::run_messages::PushRunStart,
};
use hdf5::{Dataset, Group};
use source::Source;

/// Field names for [Instrument].
mod labels {
    pub(super) const NAME: &str = "name";
    pub(super) const SOURCE: &str = "source";
}

/// Contains details about the instrument used to probe the sample.
pub(crate) struct Instrument {
    /// Name of the instrument.
    name: Dataset,
    /// The particle beam source used to probe the sample.
    _source: NexusGroup<Source>,
}

impl NexusSchematic for Instrument {
    /// The nexus class of this group.
    const CLASS: NexusClass = NexusClass::Instrument;

    /// This group structure doesn't require any settings when built.
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
    /// Sets the name of the instrument from a `RunStart` message.
    /// # Error Modes
    /// Emits [FlatBufferMissingError::InstrumentName] error if the `RunStart` message is missing the instrument name.
    /// - Propagates [Dataset::set_string] errors.
    fn handle_message(
        &mut self,
        PushRunStart(run_start): &PushRunStart<'_>,
    ) -> NexusHDF5Result<()> {
        self.name.set_string(
            run_start
                .instrument_name()
                .ok_or(FlatBufferMissingError::InstrumentName)?,
        )
    }
}
