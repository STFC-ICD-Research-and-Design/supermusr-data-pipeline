use hdf5::{types::VarLenUnicode, Attribute, Dataset, H5Type};
use ndarray::s;

use crate::{nexus::NexusDateTime, NexusWriterResult};

use super::{error::{ConvertResult, NexusHDF5Result}, DatasetExt, HasAttributesExt};

impl HasAttributesExt for Dataset {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusHDF5Result<Attribute> {
        let attr = self
            .new_attr::<VarLenUnicode>()
            .create(attr)
            .err_dataset(self)?;
        attr.write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)?;
        Ok(attr)
    }

    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        self.attr(attr).err_dataset(self)
    }
}

impl DatasetExt for Dataset {
    fn set_scalar_to<T: H5Type>(&self, value: &T) -> NexusHDF5Result<()> {
        self.write_scalar(value).err_dataset(self)
    }

    fn get_scalar_from<T: H5Type>(&self) -> NexusHDF5Result<T> {
        self.read_scalar().err_dataset(self)
    }

    fn set_string_to(&self, value: &str) -> NexusHDF5Result<()> {
        self.write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)
    }

    fn get_string_from(&self) -> NexusHDF5Result<String> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        Ok(string.into())
    }

    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        string.parse().err_dataset(self)
    }

    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()> {
        self.resize(value.len()).err_dataset(self)?;
        self.write_raw(value).err_dataset(self)
    }

    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()> {
        let cur_size = self.size();
        let new_size = cur_size + value.len();
        self.resize(new_size).err_dataset(self)?;
        self.write_slice(value, s![cur_size..new_size])
            .err_dataset(self)
    }
}
