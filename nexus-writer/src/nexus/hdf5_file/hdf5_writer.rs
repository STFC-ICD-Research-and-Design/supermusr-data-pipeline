use chrono::{DateTime, Utc};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor, VarLenUnicode},
    Attribute, Dataset, DatasetBuilderEmpty, Group, H5Type, SimpleExtents,
};
use ndarray::s;

use super::error::{ConvertResult, NexusWriterErrorType, NexusWriterResult};

pub(crate) trait HasAttributesExt {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusWriterResult<()>;
    fn get_attribute(&self, attr: &str) -> NexusWriterResult<Attribute>;
}

pub(crate) trait GroupExt {
    fn add_new_group_to(&self, name: &str, class: &str) -> NexusWriterResult<Group>;
    fn set_nx_class(&self, class: &str) -> NexusWriterResult<()>;
    fn create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset>;
    fn create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset>;

    fn create_scalar_dataset<T: H5Type>(&self, name: &str) -> NexusWriterResult<Dataset>;

    fn get_dataset(&self, name: &str) -> NexusWriterResult<Dataset>;
    fn get_dataset_or_create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset>;
    fn get_dataset_or_create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset>;
    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusWriterResult<Dataset>
    where
        F: Fn(&Group) -> NexusWriterResult<Dataset>;

    fn get_group(&self, name: &str) -> NexusWriterResult<Group>;
    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusWriterResult<Group>;
}

pub(crate) trait AttributeExt {
    fn get_datetime_from(&self) -> NexusWriterResult<DateTime<Utc>>;
}

impl AttributeExt for Attribute {
    fn get_datetime_from(&self) -> NexusWriterResult<DateTime<Utc>> {
        let string: VarLenUnicode = self.read_scalar().err_attribute(self)?;
        Ok(string.parse().err_attribute(self)?)
    }
}

impl HasAttributesExt for Group {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusWriterResult<()> {
        self.new_attr::<VarLenUnicode>()
            .create(attr)
            .err_group(self)?
            .write_scalar(&value.parse::<VarLenUnicode>().err_group(self)?)
            .err_group(self)?;
        Ok(())
    }

    fn get_attribute(&self, attr: &str) -> NexusWriterResult<Attribute> {
        Ok(self.attr(attr).err_group(self)?)
    }
}

fn get_dataset_builder(
    type_descriptor: &TypeDescriptor,
    parent: &Group,
) -> Result<DatasetBuilderEmpty, NexusWriterErrorType> {
    Ok(match type_descriptor {
        TypeDescriptor::Integer(sz) => match sz {
            IntSize::U1 => parent.new_dataset::<i8>(),
            IntSize::U2 => parent.new_dataset::<i16>(),
            IntSize::U4 => parent.new_dataset::<i32>(),
            IntSize::U8 => parent.new_dataset::<i64>(),
        },
        TypeDescriptor::Unsigned(sz) => match sz {
            IntSize::U1 => parent.new_dataset::<u8>(),
            IntSize::U2 => parent.new_dataset::<u16>(),
            IntSize::U4 => parent.new_dataset::<u32>(),
            IntSize::U8 => parent.new_dataset::<u64>(),
        },
        TypeDescriptor::Float(sz) => match sz {
            FloatSize::U4 => parent.new_dataset::<f32>(),
            FloatSize::U8 => parent.new_dataset::<f64>(),
        },
        _ => {
            return Err(NexusWriterErrorType::InvalidHDF5Type(type_descriptor.clone()))
        }
    })
}

impl GroupExt for Group {
    fn add_new_group_to(&self, name: &str, class: &str) -> NexusWriterResult<Group> {
        let group = self.create_group(name).err_group(self)?;
        group.set_nx_class(class)?;
        Ok(group)
    }

    fn set_nx_class(&self, class: &str) -> NexusWriterResult<()> {
        self.add_attribute_to("NX_class", class)
    }

    fn create_scalar_dataset<T: H5Type>(&self, name: &str) -> NexusWriterResult<Dataset> {
        Ok(self.new_dataset::<T>().create(name).err_group(self)?)
    }

    fn create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset> {
        Ok(self
            .new_dataset::<T>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![chunk_size])
            .create(name)
            .err_group(self)?)
    }

    fn create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset> {
        Ok(get_dataset_builder(type_descriptor, self)
            .err_group(self)?
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(chunk_size)
            .create(name)
            .err_group(self)?)
    }

    fn get_dataset(&self, name: &str) -> NexusWriterResult<Dataset> {
        Ok(self.dataset(name).err_group(self)?)
    }

    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusWriterResult<Dataset>
    where
        F: Fn(&Group) -> NexusWriterResult<Dataset>,
    {
        Ok(self.dataset(name).or_else(|_| f(self))?)
    }

    fn get_dataset_or_create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset> {
        Ok(self
            .dataset(name)
            .or_else(|_| self.create_resizable_empty_dataset::<T>(name, chunk_size))?)
    }

    fn get_dataset_or_create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusWriterResult<Dataset> {
        Ok(self.dataset(name).or_else(|_| {
            self.create_dynamic_resizable_empty_dataset(name, type_descriptor, chunk_size)
        })?)
    }

    fn get_group(&self, name: &str) -> NexusWriterResult<Group> {
        self.group(name).err_group(self)
    }

    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusWriterResult<Group> {
        Ok(self
            .group(name)
            .or_else(|_| self.add_new_group_to(name, class))?)
    }
}

impl HasAttributesExt for Dataset {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusWriterResult<()> {
        self.new_attr::<VarLenUnicode>()
            .create(attr)
            .err_dataset(self)?
            .write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)?;
        Ok(())
    }

    fn get_attribute(&self, attr: &str) -> NexusWriterResult<Attribute> {
        Ok(self.attr(attr).err_dataset(self)?)
    }
}

pub(crate) trait DatasetExt {
    fn set_scalar_to<T: H5Type>(&self, value: &T) -> NexusWriterResult<()>;
    fn set_string_to(&self, value: &str) -> NexusWriterResult<()>;
    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> NexusWriterResult<()>;
    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusWriterResult<()>;
    fn get_datetime_from(&self) -> NexusWriterResult<DateTime<Utc>>;
}

impl DatasetExt for Dataset {
    fn set_scalar_to<T: H5Type>(&self, value: &T) -> NexusWriterResult<()> {
        Ok(self.write_scalar(value).err_dataset(self)?)
    }

    fn set_string_to(&self, value: &str) -> NexusWriterResult<()> {
        Ok(self.write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?).err_dataset(self)?)
    }

    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> NexusWriterResult<()> {
        self.resize(value.len()).err_dataset(self)?;
        Ok(self.write_raw(value).err_dataset(self)?)
    }

    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusWriterResult<()> {
        let cur_size = self.size();
        let new_size = cur_size + value.len();
        self.resize(new_size).err_dataset(self)?;
        Ok(self.write_slice(value, s![cur_size..new_size]).err_dataset(self)?)
    }
    
    fn get_datetime_from(&self) -> NexusWriterResult<DateTime<Utc>> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        Ok(string.parse().err_dataset(self)?)
    }
}
/*
pub(super) fn add_new_group_to(parent: &Group, name: &str, class: &str) -> anyhow::Result<Group> {
    let group = parent.create_group(name)?;
    set_group_nx_class(&group, class)?;
    Ok(group)
}

pub(super) fn add_attribute_to(parent: &Location, attr: &str, value: &str) -> anyhow::Result<()> {
    parent
        .new_attr::<VarLenUnicode>()
        .create(attr)?
        .write_scalar(&value.parse::<VarLenUnicode>()?)?;
    Ok(())
}

pub(super) fn set_group_nx_class(parent: &Group, class: &str) -> anyhow::Result<()> {
    add_attribute_to(parent, "NX_class", class)
}

pub(super) fn set_string_to(target: &Dataset, value: &str) -> anyhow::Result<()> {
    Ok(target.write_scalar(&value.parse::<VarLenUnicode>()?)?)
}

pub(super) fn set_slice_to<T: H5Type>(target: &Dataset, value: &[T]) -> anyhow::Result<()> {
    target.resize(value.len())?;
    Ok(target.write_raw(value)?)
}

pub(super) fn create_resizable_dataset<T: H5Type>(
    parent: &Group,
    name: &str,
    initial_size: usize,
    chunk_size: usize,
) -> anyhow::Result<Dataset> {
    Ok(parent
        .new_dataset::<T>()
        .shape(SimpleExtents::resizable(vec![initial_size]))
        .chunk(vec![chunk_size])
        .create(name)?)
}

pub(super) fn _create_resizable_2d_dataset<T: H5Type>(
    parent: &Group,
    name: &str,
    initial_size: (usize, usize),
    chunk_size: (usize, usize),
) -> anyhow::Result<Dataset> {
    Ok(parent
        .new_dataset::<T>()
        .shape(SimpleExtents::resizable(vec![
            initial_size.0,
            initial_size.1,
        ]))
        .chunk(vec![chunk_size.0, chunk_size.1])
        .create(name)?)
}

pub(super) fn _create_resizable_2d_dataset_dyn_type(
    parent: &Group,
    name: &str,
    hdf5_type: &TypeDescriptor,
    initial_size: (usize, usize),
    chunk_size: (usize, usize),
) -> anyhow::Result<Dataset> {
    let hdf5_type = {
        if let TypeDescriptor::VarLenArray(t) = hdf5_type {
            t
        } else {
            hdf5_type
        }
    };

    let dataset = match hdf5_type {
        TypeDescriptor::Integer(sz) => match sz {
            IntSize::U1 => parent.new_dataset::<i8>(),
            IntSize::U2 => parent.new_dataset::<i16>(),
            IntSize::U4 => parent.new_dataset::<i32>(),
            IntSize::U8 => parent.new_dataset::<i64>(),
        },
        TypeDescriptor::Unsigned(sz) => match sz {
            IntSize::U1 => parent.new_dataset::<u8>(),
            IntSize::U2 => parent.new_dataset::<u16>(),
            IntSize::U4 => parent.new_dataset::<u32>(),
            IntSize::U8 => parent.new_dataset::<u64>(),
        },
        TypeDescriptor::Float(sz) => match sz {
            hdf5::types::FloatSize::U4 => parent.new_dataset::<f32>(),
            hdf5::types::FloatSize::U8 => parent.new_dataset::<f64>(),
        },
        _ => unreachable!(),
    };
    Ok(dataset
        .shape(SimpleExtents::resizable(vec![
            initial_size.0,
            initial_size.1,
        ]))
        .chunk(vec![chunk_size.0, chunk_size.1])
        .create(name)?)
}
 */
