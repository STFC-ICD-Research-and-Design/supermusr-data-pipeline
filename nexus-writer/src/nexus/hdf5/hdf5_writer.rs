use anyhow::Result;
use hdf5::{types::VarLenUnicode, Dataset, Group, H5Type, Location, SimpleExtents};

pub(super) fn add_new_group_to(parent: &Group, name: &str, class: &str) -> Result<Group> {
    let group = parent.create_group(name)?;
    set_group_nx_class(&group, class)?;
    Ok(group)
}

pub(super) fn add_attribute_to(parent: &Location, attr: &str, value: &str) -> Result<()> {
    parent
        .new_attr::<VarLenUnicode>()
        .create(attr)?
        .write_scalar(&value.parse::<VarLenUnicode>()?)?;
    Ok(())
}

pub(super) fn set_group_nx_class(parent: &Group, class: &str) -> Result<()> {
    add_attribute_to(parent, "NX_class", class)
}

pub(super) fn set_string_to(target: &Dataset, value: &str) -> Result<()> {
    Ok(target.write_scalar(&value.parse::<VarLenUnicode>()?)?)
}

pub(super) fn set_slice_to<T: H5Type>(target: &Dataset, value: &[T]) -> Result<()> {
    target.resize(value.len())?;
    Ok(target.write_raw(value)?)
}

pub(super) fn create_resizable_dataset<T: H5Type>(
    parent: &Group,
    name: &str,
    initial_size: usize,
    chunk_size: usize,
) -> Result<Dataset> {
    Ok(parent
        .new_dataset::<T>()
        .shape(SimpleExtents::resizable(vec![initial_size]))
        .chunk(vec![chunk_size])
        .create(name)?)
}
