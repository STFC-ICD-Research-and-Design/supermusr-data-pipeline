
use std::{path::{PathBuf, Path}, fmt::Display};

use hdf5::{file::File, H5Type, Extents, Group, SimpleExtents};
use anyhow::{anyhow, Result};

mod builder;
pub(crate) use builder::Nexus;

pub(crate) fn add_classed_group_to_group(parent : &Group, name: &str, class: &str) -> Result<Group> {
    let mut group = parent.create_group(name)?;
    add_nx_class_to_group(&group, class);
    Ok(group)
}

pub(crate) fn add_field_to_group<T : H5Type + Display + Copy> (parent : &Group, name: &str, content: T) -> Result<()> {
    match parent.new_dataset_builder().with_data(&[content]).create(name) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Could not add field: {name}={content} to {0}. Error: {e}",parent.name()))
    }
}
pub(crate) fn add_string_field_to_group (parent : &Group, name: &str, content: &str) -> Result<()> {
    match parent.new_dataset_builder().with_data(&[content.parse::<hdf5::types::VarLenUnicode>()?]).create(name) {
        Ok(data) => Ok(()),
        Err(e) => Err(anyhow!("Could not add string field: {name}={content} to {0}. Error: {e}",parent.name()))
    }
}
pub(crate) fn add_slice_field_to_group<T : H5Type> (parent : &Group, name: &str, content: &[T]) -> Result<()> {
    match parent.new_dataset_builder().with_data(content).create(name) {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow!("Could not add slice: {name}=[...] to {0}. Error: {e}",parent.name()))
    }
}
fn add_nx_class_to_group(parent : &Group, class: &str) -> Result<()> {
    add_attribute_to_group(parent, "NX_class", class)
}

fn add_attribute_to_group(parent : &Group, attr: &str, value: &str) -> Result<()> {
    parent.new_attr_builder().with_data(value).create(attr)?;
    Ok(())
}
