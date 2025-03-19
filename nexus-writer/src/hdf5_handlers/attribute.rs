use hdf5::{Attribute, types::VarLenUnicode};

use crate::{nexus::NexusDateTime, NexusWriterResult};

use super::{error::{ConvertResult, NexusHDF5Result}, AttributeExt};

impl AttributeExt for Attribute {
    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_attribute(self)?;
        string.parse().err_attribute(self)
    }
}
