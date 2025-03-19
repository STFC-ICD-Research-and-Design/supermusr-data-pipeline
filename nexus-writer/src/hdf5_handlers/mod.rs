mod group;
mod dataset;
mod attribute;

use hdf5::{types::VarLenUnicode, Attribute, Dataset, Group, Location};

use crate::{nexus::{DatasetExt, GroupExt, HasAttributesExt, NexusWriterError}, NexusWriterResult};


pub(crate) trait NexusSchematic : Sized {
    const CLASS: &str;

    fn create_and_setup_group(parent: &Group, name: &str) -> NexusWriterResult<Group> {
        Ok(parent.add_new_group_to(name, Self::CLASS)?)
    }
    fn build_group_structure(parent: &Group) -> NexusWriterResult<Self>;
    fn populate_group_structure(parent: &Group) -> NexusWriterResult<Self>;

    fn build_new_group(parent: &Group, name: &str) -> NexusWriterResult<NexusGroup<Self>>
    {
        let group = Self::create_and_setup_group(parent, name)?;

        let schematic = Self::build_group_structure(&group)?;

        Ok(NexusGroup::<Self> {
            group,
            schematic
        })
    }

    fn open_group(parent: &Group, name: &str) -> NexusWriterResult<NexusGroup<Self>>
    {
        let group = parent.get_group(name)?;

        let schematic = Self::populate_group_structure(&group)?;

        Ok(NexusGroup::<Self> {
            group,
            schematic
        })
    }
    fn close_group() -> NexusWriterResult<()>;
}

pub(crate) struct NexusGroup<S : NexusSchematic> {
    group: Group,
    schematic: S,
}






pub(crate) trait HasAttributesExt {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusHDF5Result<Attribute>;
    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute>;
}

pub(crate) trait GroupExt {
    fn add_new_group_to(&self, name: &str, class: &str) -> NexusHDF5Result<Group>;
    fn set_nx_class(&self, class: &str) -> NexusHDF5Result<()>;
    fn create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset>;
    fn create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset>;
    fn create_scalar_dataset<T: H5Type>(&self, name: &str) -> NexusHDF5Result<Dataset>;
    fn create_constant_scalar_dataset<T: H5Type>(&self, name: &str, value: &T) -> NexusHDF5Result<Dataset>;
    fn create_constant_string_dataset(&self, name: &str, value: &str) -> NexusHDF5Result<Dataset>;
    fn get_dataset(&self, name: &str) -> NexusHDF5Result<Dataset>;
    fn get_dataset_or_create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset>;
    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusHDF5Result<Dataset>
    where
        F: Fn(&Group) -> NexusHDF5Result<Dataset>;
    fn get_group(&self, name: &str) -> NexusHDF5Result<Group>;
    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusHDF5Result<Group>;
}

pub(crate) trait AttributeExt {
    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime>;
}