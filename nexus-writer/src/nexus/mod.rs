use std::fmt::Display;
use anyhow::{anyhow, Result};
use hdf5::{
    types::VarLenUnicode,
    H5Type,
    Group,
    Location
};

mod builder;
pub(crate) use builder::Nexus;

type AttributeList<'a> = &'a [(&'static str,&'static str)];


fn add_attribute_to(parent : &Location, attr: &str, value: &str) -> Result<()> {
    parent.new_attr_builder().with_data(&[value.parse::<VarLenUnicode>()?]).create(attr)?;
    Ok(())
}

pub(crate) fn add_attribute_list_to(field : &Location, attrs: AttributeList) -> Result<()> {
    for (attr,value) in attrs {
        add_attribute_to(field, attr,value)?;
    }
    Ok(())
}

fn set_group_nx_class(parent : &Group, class: &str) -> Result<()> {
    add_attribute_to(parent, "NX_class", class)
}

pub(crate) fn add_new_group_to(parent : &Group, name: &str, class: &str) -> Result<Group> {
    let group = parent.create_group(name)?;
    set_group_nx_class(&group, class)?;
    Ok(group)
}

pub(crate) fn add_new_field_to<T : H5Type + Display + Copy>(parent : &Group, name: &str, content: T, attrs : AttributeList) -> Result<()> {
    match parent.new_dataset_builder().with_data(&[content]).create(name) {
        Ok(field) => add_attribute_list_to(&field, attrs),
        Err(e) => Err(anyhow!("Could not add field: {name}={content} to {0}. Error: {e}",parent.name()))
    }
}

pub(crate) fn add_new_string_field_to(parent : &Group, name: &str, content: &str, attrs : AttributeList) -> Result<()> {
    match parent.new_dataset_builder().with_data(&[content.parse::<VarLenUnicode>()?]).create(name) {
        Ok(field) => add_attribute_list_to(&field, attrs),
        Err(e) => Err(anyhow!("Could not add string field: {name}={content} to {0}. Error: {e}",parent.name()))
    }
}

pub(crate) fn add_new_slice_field_to<T : H5Type>(parent : &Group, name: &str, content: &[T], attrs : AttributeList) -> Result<()> {
    match parent.new_dataset_builder().with_data(content).create(name) {
        Ok(field) => add_attribute_list_to(&field, attrs),
        Err(e) => Err(anyhow!("Could not add slice: {name}=[...] to {0}. Error: {e}",parent.name()))
    }
}
