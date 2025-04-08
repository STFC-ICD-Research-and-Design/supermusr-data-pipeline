use hdf5::{Dataset, Group};

use crate::{
    hdf5_handlers::{AttributeExt, DatasetExt, GroupExt, HasAttributesExt, NexusHDF5Result},
    nexus::NexusClass,
    nexus_structure::{NexusMessageHandler, NexusSchematic},
    run_engine::{run_messages::UpdatePeriodList, ChunkSizeSettings},
};

/// Names of datasets/attribute and subgroups in the Entry struct
mod labels {
    pub(super) const DESCRIPTION: &str = "description";
    pub(super) const COMPONENT_INDEX: &str = "component_index";
}

pub(crate) struct Geometry {
    description: Dataset,
    component_index: Dataset,
}

impl NexusSchematic for Geometry {
    const CLASS: NexusClass = NexusClass::Geometry;
    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, _settings: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            description: group.create_string_dataset(labels::DESCRIPTION)?,
            component_index: group.create_scalar_dataset::<i32>(labels::COMPONENT_INDEX)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            description: group.get_dataset(labels::DESCRIPTION)?,
            component_index: group.get_dataset(labels::COMPONENT_INDEX)?,
        })
    }
}
