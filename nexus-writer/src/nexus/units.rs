use hdf5::Dataset;

use crate::{hdf5_handlers::NexusHDF5Result, run_engine::{DatasetExt, HasAttributesExt}};

pub(crate) mod units {
    const NANOSECONDS : &str = "ns";
}

pub(crate) trait DatasetUnitExt : DatasetExt {
    fn with_units(self, units: &str) -> NexusHDF5Result<Dataset>;
}

impl DatasetUnitExt for Dataset {
    fn with_units(self, units: &str) -> NexusHDF5Result<Dataset> {
        self.add_attribute_to("units", units)?;
        Ok(self)
    }
}