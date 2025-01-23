use chrono::{DateTime, Utc};
use hdf5::{
    types::{IntSize, TypeDescriptor, VarLenUnicode}, Attribute, Dataset, Group, H5Type, Location, SimpleExtents
};
use ndarray::s;

pub(crate) trait HasAttributesExt {
    fn add_attribute_to(&self, attr: &str, value: &str) -> anyhow::Result<()>;
    fn get_attribute(&self, attr: &str) -> anyhow::Result<Attribute>;
}

pub(crate) trait GroupExt {
    fn add_new_group_to(&self, name: &str, class: &str) -> anyhow::Result<Group>;
    fn set_nx_class(&self, class: &str) -> anyhow::Result<()>;
    fn create_resizable_dataset<T: H5Type>(
        &self,
        name: &str,
        initial_size: usize,
        chunk_size: usize,
    ) -> anyhow::Result<Dataset>;

    fn get_dataset(&self, name: &str) -> anyhow::Result<Dataset>;
    fn get_group(&self, name: &str) -> anyhow::Result<Group>;
}

pub(crate) trait AttributeExt {
    fn get_datetime_from(&self) -> anyhow::Result<DateTime<Utc>>;
}

impl AttributeExt for Attribute {
    fn get_datetime_from(&self) -> anyhow::Result<DateTime<Utc>> {
        let string : VarLenUnicode = self.read_scalar()?;
        Ok(string.parse()?)
    }
}

impl HasAttributesExt for Group {
    fn add_attribute_to(&self, attr: &str, value: &str) -> anyhow::Result<()> {
        self.new_attr::<VarLenUnicode>()
            .create(attr)?
            .write_scalar(&value.parse::<VarLenUnicode>()?)?;
        Ok(())
    }
    
    fn get_attribute(&self, attr: &str) -> anyhow::Result<Attribute> {
        Ok(self.attr(attr)?)
    }
}

impl GroupExt for Group {
    fn add_new_group_to(&self, name: &str, class: &str) -> anyhow::Result<Group> {
        let group = self.create_group(name)?;
        group.set_nx_class(class)?;
        Ok(group)
    }

    fn set_nx_class(&self, class: &str) -> anyhow::Result<()> {
        self.add_attribute_to("NX_class", class)
    }

    fn create_resizable_dataset<T: H5Type>(
        &self,
        name: &str,
        initial_size: usize,
        chunk_size: usize,
    ) -> anyhow::Result<Dataset> {
        Ok(self
            .new_dataset::<T>()
            .shape(SimpleExtents::resizable(vec![initial_size]))
            .chunk(vec![chunk_size])
            .create(name)?)
    }

    fn get_dataset(&self, name: &str) -> anyhow::Result<Dataset> {
        Ok(self.dataset(name)?)
    }

    fn get_group(&self, name: &str) -> anyhow::Result<Group> {
        Ok(self.group(name)?)
    }
}

impl HasAttributesExt for Dataset {
    fn add_attribute_to(&self, attr: &str, value: &str) -> anyhow::Result<()> {
        self.new_attr::<VarLenUnicode>()
            .create(attr)?
            .write_scalar(&value.parse::<VarLenUnicode>()?)?;
        Ok(())
    }
    
    fn get_attribute(&self, attr: &str) -> anyhow::Result<Attribute> {
        Ok(self.attr(attr)?)
    }
}

pub(crate) trait DatasetExt {
    fn set_string_to(&self, value: &str) -> anyhow::Result<()>;
    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> anyhow::Result<()>;
    fn append_slice<T: H5Type>(&self, value: &[T]) -> anyhow::Result<()>;
}

impl DatasetExt for Dataset {
    fn set_string_to(&self, value: &str) -> anyhow::Result<()> {
        Ok(self.write_scalar(&value.parse::<VarLenUnicode>()?)?)
    }

    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> anyhow::Result<()> {
        self.resize(value.len())?;
        Ok(self.write_raw(value)?)
    }

    fn append_slice<T: H5Type>(&self, value: &[T]) -> anyhow::Result<()> {
        let cur_size = self.size();
        let new_size = cur_size + value.len();
        self.resize(new_size)?;
        Ok(self.write_slice(value, s![cur_size..new_size])?)
    }
}

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
