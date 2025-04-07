use super::{
    error::{ConvertResult, NexusHDF5Result},
    AttributeExt,
};
use crate::run_engine::NexusDateTime;
use hdf5::{types::VarLenUnicode, Attribute};

impl AttributeExt for Attribute {
    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_attribute(self)?;
        string.parse().err_attribute(self)
    }

    fn set_string_to(&self, value: &str) -> NexusHDF5Result<()> {
        self.write_scalar(&value.parse::<VarLenUnicode>().err_attribute(self)?)
            .err_attribute(self)
    }

    fn get_string(&self) -> NexusHDF5Result<String> {
        Ok(self
            .read_scalar::<VarLenUnicode>()
            .err_attribute(self)?
            .to_string())
    }
}
