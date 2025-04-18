mod attribute;
mod dataset;
mod dataset_flatbuffers;
mod error;
mod group;

use crate::run_engine::NexusDateTime;
pub(crate) use error::{ConvertResult, NexusHDF5Error, NexusHDF5Result};
use hdf5::{types::TypeDescriptor, Attribute, Dataset, Group, H5Type};
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::f144_LogData, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

pub(crate) trait HasAttributesExt: Sized {
    fn add_attribute<T: H5Type>(&self, attr: &str) -> NexusHDF5Result<Attribute>;
    fn add_string_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute>;
    fn add_constant_string_attribute(&self, attr: &str, value: &str) -> NexusHDF5Result<Attribute>;

    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute>;

    fn with_attribute<T: H5Type>(self, attr: &str) -> NexusHDF5Result<Self> {
        self.add_attribute::<T>(attr)?;
        Ok(self)
    }

    fn with_string_attribute(self, attr: &str) -> NexusHDF5Result<Self> {
        self.add_string_attribute(attr)?;
        Ok(self)
    }

    fn with_constant_string_attribute(self, attr: &str, value: &str) -> NexusHDF5Result<Self> {
        self.add_constant_string_attribute(attr, value)?;
        Ok(self)
    }
}

pub(crate) trait GroupExt {
    fn add_new_group(&self, name: &str, class: &str) -> NexusHDF5Result<Group>;
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
    fn create_string_dataset(&self, name: &str) -> NexusHDF5Result<Dataset>;
    fn create_constant_scalar_dataset<T: H5Type>(
        &self,
        name: &str,
        value: &T,
    ) -> NexusHDF5Result<Dataset>;
    fn create_constant_string_dataset(&self, name: &str, value: &str) -> NexusHDF5Result<Dataset>;
    fn get_dataset(&self, name: &str) -> NexusHDF5Result<Dataset>;
    #[cfg(test)]
    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusHDF5Result<Dataset>
    where
        F: Fn(&Group) -> NexusHDF5Result<Dataset>;
    fn get_group(&self, name: &str) -> NexusHDF5Result<Group>;
    #[cfg(test)]
    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusHDF5Result<Group>;
}

pub(crate) trait DatasetExt {
    fn set_scalar<T: H5Type>(&self, value: &T) -> NexusHDF5Result<()>;
    fn set_string(&self, value: &str) -> NexusHDF5Result<()>;
    fn set_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()>;

    fn append_value<T: H5Type>(&self, value: T) -> NexusHDF5Result<()>;
    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()>;

    fn get_string(&self) -> NexusHDF5Result<String>;
    fn get_datetime(&self) -> NexusHDF5Result<NexusDateTime>;
}

pub(crate) trait DatasetFlatbuffersExt {
    fn append_f144_value_slice(&self, data: &f144_LogData<'_>) -> NexusHDF5Result<()>;
    fn append_se00_value_slice(&self, data: &se00_SampleEnvironmentData<'_>)
        -> NexusHDF5Result<()>;
}

pub(crate) trait AttributeExt {
    fn set_string(&self, value: &str) -> NexusHDF5Result<()>;

    fn get_datetime(&self) -> NexusHDF5Result<NexusDateTime>;
    fn get_string(&self) -> NexusHDF5Result<String>;
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, ops::Deref, path::PathBuf};

    use super::*;

    // Helper struct to create and tidy-up a temp hdf5 file
    struct OneTempFile(Option<hdf5::File>, PathBuf);
    // Suitably long temp file name, unlikely to clash with anything else
    const TEMP_FILE_PREFIX: &str = "temp_supermusr_pipeline_nexus_writer_file";

    impl OneTempFile {
        //  We need a different file for each test, so they can run in parallel
        fn new(test_name: &str) -> Self {
            let mut path = temp_dir();
            path.push(format!("{TEMP_FILE_PREFIX}_{test_name}.nxs"));
            Self(Some(hdf5::File::create(&path).unwrap()), path)
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

        const EXPECTED_ERR_MSG : &str = "HDF5 Error: H5Gopen2(): unable to synchronously open group: object 'non_existant_group' doesn't exist at /";
        assert_eq!(maybe_group.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }

    #[test]
    fn open_nonexistant_dataset() {
        let file = OneTempFile::new("open_nonexistant_dataset");
        let maybe_dataset = file.get_dataset("non_existant_dataset");

        assert!(maybe_dataset.is_err());

        const EXPECTED_ERR_MSG : &str = "HDF5 Error: H5Dopen2(): unable to synchronously open dataset: object 'non_existant_dataset' doesn't exist at /";
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

        const EXPECTED_ERR_MSG : &str = "HDF5 Error: H5Dopen2(): unable to synchronously open dataset: object 'my_subgroup' doesn't exist at /my_group";
        assert_eq!(maybe_subgroup.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }

    #[test]
    fn open_nonexistant_attribute() {
        let file = OneTempFile::new("open_nonexistant_attribute");
        let maybe_dataset = file.get_attribute("non_existant_attribute");

        assert!(maybe_dataset.is_err());

        const EXPECTED_ERR_MSG : &str = "HDF5 Error: H5Aopen(): unable to synchronously open attribute: can't locate attribute: 'non_existant_attribute' at /";
        assert_eq!(maybe_dataset.err().unwrap().to_string(), EXPECTED_ERR_MSG);
    }
}
