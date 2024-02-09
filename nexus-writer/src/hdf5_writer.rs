use anyhow::{anyhow, Result};
use hdf5::{types::VarLenUnicode, Group, H5Type, Location, Dataset};
use std::fmt::Display;

type AttributeList<'a, 'b> = &'a [(&'static str, &'b str)];

fn add_attribute_to(parent: &Location, attr: &str, value: &str) -> Result<()> {
    parent
        .new_attr::<VarLenUnicode>()
        .create(attr)?
        .write_scalar(&value.parse::<VarLenUnicode>()?)?;
    Ok(())
}

pub(crate) fn set_attribute_list_to(field: &Location, attrs: AttributeList) -> Result<()> {
    for (attr, value) in attrs {
        add_attribute_to(field, attr, value)?;
    }
    Ok(())
}

pub(crate) fn set_group_nx_class(parent: &Group, class: &str) -> Result<()> {
    add_attribute_to(parent, "NX_class", class)
}

pub(crate) fn add_new_group_to(parent: &Group, name: &str, class: &str) -> Result<Group> {
    let group = parent.create_group(name)?;
    set_group_nx_class(&group, class)?;
    Ok(group)
}

pub(crate) fn add_new_field_to<T: H5Type + Display + Copy>(
    parent: &Group,
    name: &str,
    content: T,
) -> Result<Dataset> {
    parent
        .new_dataset_builder()
        .with_data(&[content])
        .create(name)
        .map_err(|e| anyhow!(
            "Could not add field: {name}={content} to {0}. Error: {e}",
            parent.name()
        ))
}

pub(crate) fn add_new_string_field_to(
    parent: &Group,
    name: &str,
    content: &str
) -> Result<Dataset> {
    parent
        .new_dataset_builder()
        .with_data(&[content.parse::<VarLenUnicode>()?])
        .create(name)
        .map_err(|e| anyhow!(
            "Could not add string field: {name}={content} to {0}. Error: {e}",
            parent.name()
        ))
}

pub(crate) fn add_new_slice_field_to<T: H5Type>(
    parent: &Group,
    name: &str,
    content: &[T],
) -> Result<Dataset> {
    parent
        .new_dataset_builder()
        .with_data(content)
        .create(name)
        .map_err(|e| anyhow!(
            "Could not add slice: {name}=[...] to {0}. Error: {e}",
            parent.name()
        ))
}

#[cfg(test)]
mod test {
    //use super::*;

    #[test]
    fn file_attribute_null() {
        //let file = hdf5::FileBuilder::new().create("temp1.nxs").unwrap();
        //add_attribute_to(&file, "", "").unwrap();
        //assert_eq!(file.attr_names().unwrap(), vec![""]);
        //assert_eq!(file.attr("").unwrap().unwrap(), Datatype::from_descriptor(&hdf5::types::TypeDescriptor::VarLenUnicode).unwrap());
    }

    #[test]
    fn file_attribute_test() {
        //let file = hdf5::FileBuilder::new().create("temp1.nxs").unwrap();
        //add_attribute_to(&file, "", "").unwrap();
        //assert_eq!(file.attr_names().unwrap(), vec![""]);
    }
}
