mod geometry;

use geometry::Geometry;
use hdf5::{Dataset, Group};

use crate::{
    hdf5_handlers::{GroupExt, NexusHDF5Result},
    nexus::{DatasetUnitExt, NexusClass, NexusGroup, NexusUnits},
    nexus_structure::NexusSchematic,
    run_engine::ChunkSizeSettings,
};

mod labels {
    pub(super) const NAME: &str = "name";
    pub(super) const DESCRIPTION: &str = "description";
    pub(super) const SAMPLE_TYPE: &str = "type";
    pub(super) const GEOMETRY: &str = "geometry";
    pub(super) const THICKNESS: &str = "thickness";
    pub(super) const MASS: &str = "mass";
    pub(super) const DENSITY: &str = "density";
    pub(super) const TEMPERATURE: &str = "temperature";
    pub(super) const MAGNETIC_FIELD: &str = "magnetic_field";
}

/// Names of datasets/attribute and subgroups in the Entry struct
pub(crate) struct Sample {
    name: Dataset,
    description: Dataset,
    sample_type: Dataset,
    geometry: NexusGroup<Geometry>,
    thickness: Dataset,
    mass: Dataset,
    density: Dataset,
    temperature: Dataset,
    magnetic_field: Dataset,
}

impl NexusSchematic for Sample {
    const CLASS: NexusClass = NexusClass::Sample;
    type Settings = ChunkSizeSettings;

    fn build_group_structure(group: &Group, settings: &Self::Settings) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.create_string_dataset(labels::NAME)?,
            description: group.create_string_dataset(labels::DESCRIPTION)?,
            sample_type: group.create_string_dataset(labels::SAMPLE_TYPE)?,
            geometry: Geometry::build_new_group(group, labels::GEOMETRY, settings)?,
            thickness: group
                .create_resizable_empty_dataset::<f32>(labels::THICKNESS, settings.period)?
                .with_units(NexusUnits::Millimeters)?,
            mass: group
                .create_resizable_empty_dataset::<f32>(labels::MASS, settings.period)?
                .with_units(NexusUnits::Milligrams)?,
            density: group
                .create_resizable_empty_dataset::<f32>(labels::DENSITY, settings.period)?
                .with_units(NexusUnits::MilligramsPerCm3)?,
            temperature: group
                .create_scalar_dataset::<f32>(labels::TEMPERATURE)?
                .with_units(NexusUnits::Kelvin)?,
            magnetic_field: group
                .create_scalar_dataset::<f32>(labels::MAGNETIC_FIELD)?
                .with_units(NexusUnits::Gauss)?,
        })
    }

    fn populate_group_structure(group: &Group) -> NexusHDF5Result<Self> {
        Ok(Self {
            name: group.get_dataset(labels::NAME)?,
            description: group.get_dataset(labels::DESCRIPTION)?,
            sample_type: group.get_dataset(labels::SAMPLE_TYPE)?,
            geometry: Geometry::open_group(group, labels::GEOMETRY)?,
            thickness: group.get_dataset(labels::THICKNESS)?,
            mass: group.get_dataset(labels::MASS)?,
            density: group.get_dataset(labels::DENSITY)?,
            temperature: group.get_dataset(labels::TEMPERATURE)?,
            magnetic_field: group.get_dataset(labels::MAGNETIC_FIELD)?,
        })
    }
}
