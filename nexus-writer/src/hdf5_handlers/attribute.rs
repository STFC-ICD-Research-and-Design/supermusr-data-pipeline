use super::{
    error::{ConvertResult, NexusHDF5Result},
    AttributeExt,
};
use crate::nexus::NexusDateTime;
use hdf5::{types::VarLenUnicode, Attribute};

impl AttributeExt for Attribute {
    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_attribute(self)?;
        string.parse().err_attribute(self)
    }
}
