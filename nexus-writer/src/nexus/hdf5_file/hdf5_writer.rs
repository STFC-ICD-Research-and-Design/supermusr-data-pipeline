use anyhow::Result;
use hdf5::{
    types::{IntSize, TypeDescriptor, VarLenUnicode},
    Dataset, Group, H5Type, Location, SimpleExtents,
};

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

pub(super) fn create_resizable_2d_dataset<T: H5Type>(
    parent: &Group,
    name: &str,
    initial_size: (usize, usize),
    chunk_size: (usize, usize),
) -> Result<Dataset> {
    Ok(parent
        .new_dataset::<T>()
        .shape(SimpleExtents::resizable(vec![
            initial_size.0,
            initial_size.1,
        ]))
        .chunk(vec![chunk_size.0, chunk_size.1])
        .create(name)?)
}

pub(super) fn create_resizable_2d_dataset_dyn_type(
    parent: &Group,
    name: &str,
    hdf5_type: &TypeDescriptor,
    initial_size: (usize, usize),
    chunk_size: (usize, usize),
) -> Result<Dataset> {
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
