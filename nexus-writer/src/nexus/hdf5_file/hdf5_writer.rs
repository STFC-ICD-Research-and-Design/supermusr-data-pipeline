use super::error::{ConvertResult, NexusHDF5ErrorType, NexusHDF5Result};
use crate::nexus::NexusDateTime;
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor, VarLenUnicode},
    Attribute, Dataset, DatasetBuilderEmpty, Group, H5Type, SimpleExtents,
};
use ndarray::s;

pub(crate) trait HasAttributesExt {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusHDF5Result<()>;
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

impl AttributeExt for Attribute {
    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_attribute(self)?;
        string.parse().err_attribute(self)
    }
}

impl HasAttributesExt for Group {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusHDF5Result<()> {
        self.new_attr::<VarLenUnicode>()
            .create(attr)
            .err_group(self)?
            .write_scalar(&value.parse::<VarLenUnicode>().err_group(self)?)
            .err_group(self)?;
        Ok(())
    }

    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        self.attr(attr).err_group(self)
    }
}

fn get_dataset_builder(
    type_descriptor: &TypeDescriptor,
    parent: &Group,
) -> Result<DatasetBuilderEmpty, NexusHDF5ErrorType> {
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
        _ => return Err(NexusHDF5ErrorType::InvalidHDF5Type(type_descriptor.clone())),
    })
}

impl GroupExt for Group {
    fn add_new_group_to(&self, name: &str, class: &str) -> NexusHDF5Result<Group> {
        let group = self.create_group(name).err_group(self)?;
        group.set_nx_class(class)?;
        Ok(group)
    }

    fn set_nx_class(&self, class: &str) -> NexusHDF5Result<()> {
        self.add_attribute_to("NX_class", class)
    }

    fn create_scalar_dataset<T: H5Type>(&self, name: &str) -> NexusHDF5Result<Dataset> {
        self.new_dataset::<T>().create(name).err_group(self)
    }

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

    fn get_dataset(&self, name: &str) -> NexusHDF5Result<Dataset> {
        self.dataset(name).err_group(self)
    }

    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusHDF5Result<Dataset>
    where
        F: Fn(&Group) -> NexusHDF5Result<Dataset>,
    {
        self.dataset(name).or_else(|_| f(self))
    }

    fn get_dataset_or_create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset> {
        self.dataset(name).or_else(|_| {
            self.create_dynamic_resizable_empty_dataset(name, type_descriptor, chunk_size)
        })
    }

    fn get_group(&self, name: &str) -> NexusHDF5Result<Group> {
        self.group(name).err_group(self)
    }

    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusHDF5Result<Group> {
        self.group(name)
            .or_else(|_| self.add_new_group_to(name, class))
    }
}

impl HasAttributesExt for Dataset {
    fn add_attribute_to(&self, attr: &str, value: &str) -> NexusHDF5Result<()> {
        self.new_attr::<VarLenUnicode>()
            .create(attr)
            .err_dataset(self)?
            .write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)?;
        Ok(())
    }

    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute> {
        self.attr(attr).err_dataset(self)
    }
}

pub(crate) trait DatasetExt {
    fn set_scalar_to<T: H5Type>(&self, value: &T) -> NexusHDF5Result<()>;
    fn get_scalar_from<T: H5Type>(&self) -> NexusHDF5Result<T>;
    fn set_string_to(&self, value: &str) -> NexusHDF5Result<()>;
    fn get_string_from(&self) -> NexusHDF5Result<String>;
    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime>;
    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()>;
    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()>;
}

impl DatasetExt for Dataset {
    fn set_scalar_to<T: H5Type>(&self, value: &T) -> NexusHDF5Result<()> {
        self.write_scalar(value).err_dataset(self)
    }

    fn get_scalar_from<T: H5Type>(&self) -> NexusHDF5Result<T> {
        self.read_scalar().err_dataset(self)
    }

    fn set_string_to(&self, value: &str) -> NexusHDF5Result<()> {
        self.write_scalar(&value.parse::<VarLenUnicode>().err_dataset(self)?)
            .err_dataset(self)
    }

    fn get_string_from(&self) -> NexusHDF5Result<String> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        Ok(string.into())
    }

    fn get_datetime_from(&self) -> NexusHDF5Result<NexusDateTime> {
        let string: VarLenUnicode = self.read_scalar().err_dataset(self)?;
        string.parse().err_dataset(self)
    }

    fn set_slice_to<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()> {
        self.resize(value.len()).err_dataset(self)?;
        self.write_raw(value).err_dataset(self)
    }

    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()> {
        let cur_size = self.size();
        let new_size = cur_size + value.len();
        self.resize(new_size).err_dataset(self)?;
        self.write_slice(value, s![cur_size..new_size])
            .err_dataset(self)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Deref;

    use super::*;

    // Helper struct to create and tidy-up a temp hdf5 file
    struct OneTempFile(Option<hdf5::File>, String);
    // Suitably long temp file name, unlikely to clash with anything else
    const TEMP_FILE_PATH: &str = "/tmp/temp_supermusr_pipeline_nexus_writer_file";

    impl OneTempFile {
        //  We need a different file for each test, so they can run in parallel
        fn new(test_name: &str) -> Self {
            let temp_file_name = format!("{TEMP_FILE_PATH}_{test_name}.nxs");
            Self(
                Some(hdf5::File::create(&temp_file_name).unwrap()),
                temp_file_name,
            )
        }
    }

    //  Cleans up the temp directory after our test
    impl Drop for OneTempFile {
        fn drop(&mut self) {
            let file = self.0.take().unwrap();
            file.close().unwrap();
            std::fs::remove_file(&self.1).unwrap();
        }
    }

    //  So we can use our OneTempFile as an hdf5 file
    impl Deref for OneTempFile {
        type Target = hdf5::File;

        fn deref(&self) -> &Self::Target {
            self.0.as_ref().unwrap()
        }
    }

    #[test]
    fn create_group() {
        let file = OneTempFile::new("create_group");
        let maybe_group = file.get_group_or_create_new("my_group", "my_class");

        assert!(maybe_group.is_ok());
        assert_eq!(maybe_group.unwrap().name().as_str(), "/my_group");
    }

    #[test]
    fn create_nested_group() {
        let file = OneTempFile::new("create_nested_group");
        let group = file
            .get_group_or_create_new("my_group", "my_class")
            .unwrap();
        let maybe_subgroup = group.get_group_or_create_new("my_subgroup", "my_subclass");

        assert!(maybe_subgroup.is_ok());
        assert_eq!(
            maybe_subgroup.unwrap().name().as_str(),
            "/my_group/my_subgroup"
        );
    }

    #[test]
    fn create_dataset() {
        let file = OneTempFile::new("create_dataset");
        let maybe_dataset = file.get_dataset_or_else("my_dataset", |group| {
            group.create_scalar_dataset::<u8>("my_dataset")
        });

        assert!(maybe_dataset.is_ok());
        assert_eq!(maybe_dataset.unwrap().name().as_str(), "/my_dataset");
    }

    #[test]
    fn open_nonexistant_group() {
        let file = OneTempFile::new("open_nonexistant_group");
        let maybe_group = file.get_group("non_existant_group");

        assert!(maybe_group.is_err());

        const EXPECTED_ERR_MSG : &str = "H5Gopen2(): unable to synchronously open group: object 'non_existant_group' doesn't exist at /";
        assert_eq!(maybe_group.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }

    #[test]
    fn open_nonexistant_dataset() {
        let file = OneTempFile::new("open_nonexistant_dataset");
        let maybe_dataset = file.get_dataset("non_existant_dataset");

        assert!(maybe_dataset.is_err());

        const EXPECTED_ERR_MSG : &str = "H5Dopen2(): unable to synchronously open dataset: object 'non_existant_dataset' doesn't exist at /";
        assert_eq!(maybe_dataset.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }

    #[test]
    fn open_nonexistant_nested_dataset() {
        let file = OneTempFile::new("open_nonexistant_nested_dataset");
        let group = file
            .get_group_or_create_new("my_group", "my_class")
            .unwrap();
        let maybe_subgroup = group.get_dataset("my_subgroup");

        assert!(maybe_subgroup.is_err());

        const EXPECTED_ERR_MSG : &str = "H5Dopen2(): unable to synchronously open dataset: object 'my_subgroup' doesn't exist at /my_group";
        assert_eq!(maybe_subgroup.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }

    #[test]
    fn open_nonexistant_attribute() {
        let file = OneTempFile::new("open_nonexistant_attribute");
        let maybe_dataset = file.get_attribute("non_existant_attribute");

        assert!(maybe_dataset.is_err());

        const EXPECTED_ERR_MSG : &str = "H5Aopen(): unable to synchronously open attribute: can't locate attribute: 'non_existant_attribute' at /";
        assert_eq!(maybe_dataset.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }
}
