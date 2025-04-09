use hdf5::{types::VarLenUnicode, Attribute, Dataset, H5Type};
use ndarray::s;

use crate::run_engine::NexusDateTime;

use super::{
    error::{ConvertResult, NexusHDF5Result},
    DatasetExt, HasAttributesExt,
};

impl HasAttributesExt for Dataset {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn add_attribute(&self, attr: &str, value: &str) -> NexusHDF5Result<Attribute> {
        let attr = self
            .new_attr::<VarLenUnicode>()
            .create(attr)
            .err_dataset(self)?;
        attr.write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)?;
        Ok(attr)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        self.attr(attr).err_dataset(self)
    }
}

impl DatasetExt for Dataset {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn set_scalar<T: H5Type>(&self, value: &T) -> NexusHDF5Result<()> {
        self.write_scalar(value).err_dataset(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_scalar<T: H5Type>(&self) -> NexusHDF5Result<T> {
        self.read_scalar().err_dataset(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn set_string(&self, value: &str) -> NexusHDF5Result<()> {
        self.write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_string(&self) -> NexusHDF5Result<String> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        Ok(string.into())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_datetime(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        string.parse().err_dataset(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn set_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()> {
        self.resize(value.len()).err_dataset(self)?;
        self.write_raw(value).err_dataset(self)
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn append_value<T: H5Type>(&self, value: T) -> NexusHDF5Result<()> {
        self.append_slice(&[value])
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()> {
        let cur_size = self.size();
        let new_size = cur_size + value.len();
        self.resize(new_size).err_dataset(self)?;
        self.write_slice(value, s![cur_size..new_size])
            .err_dataset(self)
    }
}
