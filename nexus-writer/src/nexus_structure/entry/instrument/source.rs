use hdf5::{Dataset, Group};

use super::NexusSchematic;
use crate::{
    hdf5_handlers::{GroupExt, NexusHDF5Result},
    nexus::NexusClass,
};

/// Names of datasets/attribute and subgroups in the Entry struct
mod labels {
    pub(super) const NAME: &str = "name";
    pub(super) const SOURCE_TYPE: &str = "type";
    pub(super) const PROBE: &str = "probe";
}

// Values of Nexus Constant
const NAME: &str = "ISIS";
const SOURCE_TYPE: &str = "pulsed muon source";
const PROBE: &str = "negative muons";

pub(crate) struct Source {
    _name: Dataset,
    _source_type: Dataset,
    _probe: Dataset,
}

impl NexusSchematic for Source {
    const CLASS: NexusClass = NexusClass::Source;
    type Settings = ();

    fn build_group_structure(group: &Group, _: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            _name: group.create_constant_string_dataset(labels::NAME, NAME)?,
            _source_type: group.create_constant_string_dataset(labels::SOURCE_TYPE, SOURCE_TYPE)?,
            _probe: group.create_constant_string_dataset(labels::PROBE, PROBE)?, // TODO  Is this correct?
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            _name: group.get_dataset(labels::NAME)?,
            _source_type: group.get_dataset(labels::SOURCE_TYPE)?,
            _probe: group.get_dataset(labels::PROBE)?,
        })
    }
}
