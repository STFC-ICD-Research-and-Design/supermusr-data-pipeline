//! This module defines the enum `NexusUnit` which is used by some Dataset instances
//! to implement a textual hdf5 attribute "units" which indicates the dataset is a
//! quantity of the specified units.
use crate::hdf5_handlers::{DatasetExt, HasAttributesExt, NexusHDF5Result};
use hdf5::Dataset;

#[derive(strum::Display)]
pub(crate) enum NexusUnits {
    #[strum(to_string = "Hz")]
    Hertz,
    // Time
    #[strum(to_string = "second")]
    Seconds,
    #[strum(to_string = "ms")]
    Milliseconds,
    #[strum(to_string = "ns")]
    Nanoseconds,
    // Energy
    #[strum(to_string = "eV")]
    ElectronVolts,
    #[strum(to_string = "MeV")]
    MegaElectronVolts,
    // Momentum
    #[strum(to_string = "MeVc^-1")]
    MegaElectronVoltsOverC,
    // Current
    #[strum(to_string = "uA")]
    MicroAmps,
    // Charge
    #[strum(to_string = "uAh")]
    MicroAmpHours,
    // Length
    #[strum(to_string = "mm")]
    Millimeters,
    // Mass
    #[strum(to_string = "mg")]
    Milligrams,
    // Density
    #[strum(to_string = "mgcm^-3")]
    MilligramsPerCm3,
    // Temperature
    #[strum(to_string = "K")]
    Kelvin,
    // Magnetic Field
    #[strum(to_string = "G")]
    Gauss,
}

pub(crate) trait DatasetUnitExt: DatasetExt {
    fn with_units(self, units: NexusUnits) -> NexusHDF5Result<Dataset>;
}

impl DatasetUnitExt for Dataset {
    fn with_units(self, units: NexusUnits) -> NexusHDF5Result<Dataset> {
        self.add_constant_string_attribute("units", &units.to_string())?;
        Ok(self)
    }
}
