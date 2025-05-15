//! This module implements the [GroupExt] and [HasAttributesExt] traits for
//! the hdf5 [Group] type.
use super::{
    error::{ConvertResult, NexusHDF5Error, NexusHDF5Result},
    DatasetExt, GroupExt, HasAttributesExt,
};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor, VarLenUnicode},
    Attribute, Dataset, DatasetBuilderEmpty, Group, H5Type, SimpleExtents,
};

impl HasAttributesExt for Group {
    /// Creates a new attribute, with name as specified.
    /// # Parameters
    ///  - attr: name of the attribute to add.
    /// # Error Modes
    fn add_attribute<T: H5Type>(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        let attr = self.new_attr::<T>().create(attr).err_group(self)?;
        Ok(attr)
    }

    /// This should be a provided method in the trait?
    fn add_string_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        self.add_attribute::<VarLenUnicode>(attr)
    }

    /// This should be a provided method in the trait?
    /// Creates a new string-typed attribute, with name and contents as specified.
    /// # Parameters
    ///  - attr: name of the attribute to add.
    ///  - value: content of the attribute to add.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// - Propagates errors from [Self::add_string_attribute()].
    /// - Propagates errors from [write_scalar()].
    /// - Propagates [NexusHDF5Error::HDF5String] errors.
    /// 
    /// [write_scalar()]: hdf5::Container::write_scalar()
    fn add_constant_string_attribute(&self, attr: &str, value: &str) -> NexusHDF5Result<Attribute> {
        let attr = self.add_string_attribute(attr)?;
        attr.write_scalar(&value.parse::<VarLenUnicode>().err_group(self)?)
            .err_group(self)?;
        Ok(attr)
    }

    /// Returns the attribute matching the given name.
    /// # Parameters
    ///  - attr: name of the attribute to get.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// - Propagates errors from [attr()], in particular if the attribute does not exist.
    /// 
    /// [attr()]: hdf5::Location::attr()
    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        self.attr(attr).err_group(self)
    }
}

/// Creates a hdf5 [DatasetBuilderEmpty] object with the appropriate type specified by `type_descriptor`.
/// This is only used by [create_dynamic_resizable_empty_dataset].
///
/// [create_dynamic_resizable_empty_dataset]: Group::create_dynamic_resizable_empty_dataset()
fn get_dataset_builder(
    type_descriptor: &TypeDescriptor,
    parent: &Group,
) -> NexusHDF5Result<DatasetBuilderEmpty> {
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
        TypeDescriptor::VarLenUnicode => parent.new_dataset::<VarLenUnicode>(),
        _ => {
            return Err(NexusHDF5Error::InvalidHDF5Type {
                error: type_descriptor.clone(),
                hdf5_path: None,
            })
        }
    })
}

impl GroupExt for Group {
    /// Create a new subgroup of this group, with name and class as specified.
    /// # Parameters
    ///  - name: name of the group to add.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// - Propagates errors from [create_group()].
    /// - Propagates errors from [Self::set_nx_class()].
    /// 
    /// [create_group()]: hdf5::Group::create_group()
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn add_new_group(&self, name: &str, class: &str) -> NexusHDF5Result<Group> {
        let group = self.create_group(name).err_group(self)?;
        group.set_nx_class(class)?;
        Ok(group)
    }

    /// Creates an attribute in this group named "NX_class" and contents as specified.
    /// # Parameters
    /// - class: the name of the class to add to the group.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn set_nx_class(&self, class: &str) -> NexusHDF5Result<()> {
        self.add_constant_string_attribute("NX_class", class)?;
        Ok(())
    }

    /// Creates a new scalar dataset in this group with static type `T`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    /// Implementation should try to return the new dataset.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// Propagates errors from [DatasetBuilderEmpty::create()].
    /// 
    /// [DatasetBuilderEmpty::create()]: hdf5::DatasetBuilderEmpty::create()
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_scalar_dataset<T: H5Type>(&self, name: &str) -> NexusHDF5Result<Dataset> {
        self.new_dataset::<T>().create(name).err_group(self)
    }

    /// This should be a provided function of the trait.
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_string_dataset(&self, name: &str) -> NexusHDF5Result<Dataset> {
        self.create_scalar_dataset::<VarLenUnicode>(name)
    }

    /// This should be a provided function of the trait.
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_constant_scalar_dataset<T: H5Type>(
        &self,
        name: &str,
        value: &T,
    ) -> NexusHDF5Result<Dataset> {
        let dataset = self.create_scalar_dataset::<T>(name)?;
        dataset.set_scalar(value)?;
        Ok(dataset)
    }

    /// This should be a provided function of the trait.
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_constant_string_dataset(&self, name: &str, value: &str) -> NexusHDF5Result<Dataset> {
        let dataset = self.create_scalar_dataset::<VarLenUnicode>(name)?;
        dataset.set_string(value)?;
        Ok(dataset)
    }

    /// Creates a new one-dimensional dataset in this group with static type `T`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// Propagates errors from [DatasetBuilderEmptyShape::create()].
    /// 
    /// [DatasetBuilderEmptyShape::create()]: hdf5::DatasetBuilderEmptyShape::create()
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset> {
        self.new_dataset::<T>()
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(vec![chunk_size])
            .create(name)
            .err_group(self)
    }

    /// Creates a new one-dimensional dataset in this group with type dynamically specified by `type_descriptor`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// Propagates errors from [DatasetBuilderEmptyShape::create()].
    /// 
    /// [DatasetBuilderEmptyShape::create()]: hdf5::DatasetBuilderEmptyShape::create()
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset> {
        get_dataset_builder(type_descriptor, self)
            .err_group(self)?
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(chunk_size)
            .create(name)
            .err_group(self)
    }

    /// Returns the dataset in this group matching the given name.
    /// # Parameters
    ///  - name: name of the dataset to get.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// - Propagates errors from [dataset()], in particular if the dataset does not exist.
    /// 
    /// [dataset()]: hdf5::Group::dataset()
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_dataset(&self, name: &str) -> NexusHDF5Result<Dataset> {
        self.dataset(name).err_group(self)
    }

    #[cfg(test)]
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusHDF5Result<Dataset>
    where
        F: Fn(&Group) -> NexusHDF5Result<Dataset>,
    {
        self.dataset(name).or_else(|_| f(self))
    }

    /// Returns the subgroup in this group matching the given name.
    /// # Parameters
    ///  - name: name of the group to get.
    /// # Error Modes
    /// Appends the hdf5 path to any errors.
    /// - Propagates errors from [group()], in particular if the subgroup does not exist.
    /// 
    /// [group()]: hdf5::Group::group()
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_group(&self, name: &str) -> NexusHDF5Result<Group> {
        self.group(name).err_group(self)
    }

    #[cfg(test)]
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusHDF5Result<Group> {
        self.group(name)
            .or_else(|_| self.add_new_group(name, class))
    }
}
