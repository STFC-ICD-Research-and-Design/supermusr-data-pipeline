//! This module implements the traits to extend the hdf5 [Attribute] type to provide robust, conventient helper methods.
use super::{
    AttributeExt,
    error::{ConvertResult, NexusHDF5Result},
};
use crate::run_engine::NexusDateTime;
use hdf5::{Attribute, types::VarLenUnicode};

impl AttributeExt for Attribute {
    /// Maybe this should be a provided method?
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_datetime(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_attribute(self)?;
        string.parse().err_attribute(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn set_string(&self, value: &str) -> NexusHDF5Result<()> {
        self.write_scalar(&value.parse::<VarLenUnicode>().err_attribute(self)?)
            .err_attribute(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_string(&self) -> NexusHDF5Result<String> {
        Ok(self
            .read_scalar::<VarLenUnicode>()
            .err_attribute(self)?
            .to_string())
    }
}
