//! Defines traits which extend hdf5 types [Group], [Dataset] and [Attribute],
//! to enable building NeXus files more convenient and robust.
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

/// This is implemented by hdf5 types [Group] and [Dataset], both can have attributes set
/// and this trait provides a common interface for them both.
pub(crate) trait HasAttributesExt: Sized {
    /// Implementation should create a new attribute, with name as specified.
    /// # Parameters
    ///  - attr: name of the attribute to add
    /// # Return
    /// Implementation should attempt to return the created attribute.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call the approriate `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn add_attribute<T: H5Type>(&self, attr: &str) -> NexusHDF5Result<Attribute>;
    
    /// Implementation should create a new string-typed attribute, with name and contents as specified.
    /// # Parameters
    ///  - attr: name of the attribute to add
    /// # Return
    /// Implementation should attempt to return the created attribute.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn add_string_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute>;
    
    /// Implementation should create a new string-typed attribute, with name and contents as specified.
    /// # Parameters
    ///  - attr: name of the attribute to add
    ///  - value: content of the attribute to add
    /// # Return
    /// Implementation should attempt to return the created attribute.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn add_constant_string_attribute(&self, attr: &str, value: &str) -> NexusHDF5Result<Attribute>;

    /// Implementation should return the attribute matching the given name.
    /// # Parameters
    ///  - attr: name of the attribute to get
    /// # Return
    /// Implementation should attempt to return the selected attribute.
    /// If this attribute does not exist, an error should be returned.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn get_attribute(&self, attr: &str) -> NexusHDF5Result<Attribute>;

    /// Implementation should create a new attribute, with name as specified, and return the original calling object.
    /// # Parameters
    ///  - attr: name of the attribute to add.
    /// # Return
    /// Implementation should return calling object, modified to add the attribute.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn with_attribute<T: H5Type>(self, attr: &str) -> NexusHDF5Result<Self> {
        self.add_attribute::<T>(attr)?;
        Ok(self)
    }

    /// Implementation should create a new string-typed attribute, with name as specified, and return the original calling object.
    /// # Parameters
    ///  - attr: name of the attribute to add.
    /// # Return
    /// Implementation should return calling object, modified to add the attribute.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn with_string_attribute(self, attr: &str) -> NexusHDF5Result<Self> {
        self.add_string_attribute(attr)?;
        Ok(self)
    }

    /// Implementation should create a new string-typed attribute, with name and content as specified, and return the original calling object.
    /// # Parameters
    ///  - attr: name of the attribute to add.
    /// # Return
    /// Implementation should return calling object, modified to add the attribute.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_xxx(self)` on any error
    /// to set the error's `hdf5_path` field
    fn with_constant_string_attribute(self, attr: &str, value: &str) -> NexusHDF5Result<Self> {
        self.add_constant_string_attribute(attr, value)?;
        Ok(self)
    }
}

/// Provides methods to be called on the hdf5 `Group` type.
/// These provide additional guarantees that the resulting `Group`
/// is NeXus compliant.
pub(crate) trait GroupExt {
    /// Implementations should create a new subgroup of this group, with name and class as specified.
    /// # Parameters
    ///  - name: name of the group to add.
    /// # Return
    /// Implementation should try to return the new group.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call [NexusHDF5Result::err_group] on any error
    /// to set the error's `hdf5_path` field
    fn add_new_group(&self, name: &str, class: &str) -> NexusHDF5Result<Group>;
    
    /// Implementations should create an attribute in this group named "NX_class" and contents as specified.
    /// # Parameters
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn set_nx_class(&self, class: &str) -> NexusHDF5Result<()>;

    /// Implementations should create a new one-dimensional dataset in this group with static type `T`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    /// Implementation should try to return the new dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn create_resizable_empty_dataset<T: H5Type>(
        &self,
        name: &str,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset>;
    
    /// Implementations should create a new one-dimensional dataset in this group with type dynamically specified by `type_descriptor`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    /// Implementation should try to return the new dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn create_dynamic_resizable_empty_dataset(
        &self,
        name: &str,
        type_descriptor: &TypeDescriptor,
        chunk_size: usize,
    ) -> NexusHDF5Result<Dataset>;
    
    /// Implementations should create a new scalar dataset in this group with static type `T`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    /// Implementation should try to return the new dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn create_scalar_dataset<T: H5Type>(&self, name: &str) -> NexusHDF5Result<Dataset>;
    
    /// Implementations should create a new scalar dataset in this group with static type `hdf5::VarLenUnicode`.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    /// Implementation should try to return the new dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn create_string_dataset(&self, name: &str) -> NexusHDF5Result<Dataset>;
    
    /// Implementations should create a new scalar dataset in this group with static type `T`, and contents as specified.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    /// Implementation should try to return the new dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn create_constant_scalar_dataset<T: H5Type>(
        &self,
        name: &str,
        value: &T,
    ) -> NexusHDF5Result<Dataset>;

    /// Implementations should create a new scalar dataset in this group with static type `hdf5::VarLenUnicode`, and contents as specified.
    /// # Parameters
    ///  - name: name of the dataset to add.
    /// # Return
    ///  - Implementation should try to return the new dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn create_constant_string_dataset(&self, name: &str, value: &str) -> NexusHDF5Result<Dataset>;

    /// Implementations should return the dataset in this group matching the given name.
    /// # Parameters
    ///  - name: name of the dataset to get.
    /// # Return
    ///  - Implementation should try to return the selected dataset.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call `NexusHDF5Result::err_group(self)` on any error
    /// to set the error's `hdf5_path` field
    fn get_dataset(&self, name: &str) -> NexusHDF5Result<Dataset>;

    #[cfg(test)]
    fn get_dataset_or_else<F>(&self, name: &str, f: F) -> NexusHDF5Result<Dataset>
    where
        F: Fn(&Group) -> NexusHDF5Result<Dataset>;
    
    /// Implementations should return the subgroup in this group matching the given name.
    /// # Parameters
    ///  - name: name of the group to get.
    /// # Error Modes
    /// Implementations should propagate any hdf5 errors and call 'NexusHDF5Result::err_group(self)' on any error
    /// to set the error's `hdf5_path` field
    fn get_group(&self, name: &str) -> NexusHDF5Result<Group>;
    
    #[cfg(test)]
    fn get_group_or_create_new(&self, name: &str, class: &str) -> NexusHDF5Result<Group>;
}

/// This trait provides methods to be called on the hdf5 [Dataset] type.
/// These methods provide additional guarantees that the resulting [Dataset]
/// is NeXus compliant.
pub(crate) trait DatasetExt {
    /// Implementation should set the value of the dataset to the single value at the provided reference.
    /// # Parameters
    /// - value: value to set the dataset to.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type `T`,
    ///     - is scalar,
    /// and should return an error otherwise.
    fn set_scalar<T: H5Type>(&self, value: &T) -> NexusHDF5Result<()>;

    /// Implementation should set the value of the dataset to the given string slice value.
    /// # Parameters
    /// - value: slice to set the dataset to.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type [hdf5::types::VarLenUnicode],
    ///     - is scalar,
    /// and should return an error otherwise.
    fn set_string(&self, value: &str) -> NexusHDF5Result<()>;
    
    /// Implementation should set the value of the dataset to the slice value.
    /// # Parameters
    /// - value: slice to set the dataset to.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type `T`,
    ///     - is one-dimentional,
    /// and should return an error if it was not.
    fn set_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()>;

    /// Implementation should increases the size of the dataset by one, and sets the new value to value at the provided reference.
    /// # Parameters
    /// - value: value to set the dataset to.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type `T`,
    ///     - is one-dimentional,
    /// and should return an error if it was not.
    fn append_value<T: H5Type>(&self, value: T) -> NexusHDF5Result<()>;
    
    /// Implementation should increases the size of the dataset by the size of the given slice, and sets the new values to ones in the provided slice.
    /// # Parameters
    /// - value: value to set the dataset to.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type `T`,
    ///     - is one-dimentional,
    /// and should return an error if it was not.
    fn append_slice<T: H5Type>(&self, value: &[T]) -> NexusHDF5Result<()>;

    /// Implementation should return a [String] with the contents of the dataset.
    /// # Return
    /// - `String` with the contents of the dataset.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type [hdf5::types::VarLenUnicode],
    ///     - is scalar,
    /// and should return an error if it was not.
    fn get_string(&self) -> NexusHDF5Result<String>;

    /// Implementation should return the timestamp from the dataset.
    /// # Return
    /// - The timestamp of the dataset.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type [hdf5::types::VarLenUnicode],
    ///     - is scalar,
    ///     - the contents can be parsed into a valid [NexusDateTime],
    /// and should return an error if it was not.
    fn get_datetime(&self) -> NexusHDF5Result<NexusDateTime>;
}

/// This trait provides methods to be called on the hdf5 [Dataset] type,
/// for appending data from specific flatbuffer messages.
/// These methods provide additional guarantees that the resulting [Dataset]
/// is NeXus compliant.
pub(crate) trait DatasetFlatbuffersExt {
    /// Implementation should append values from the given flatbuffer `LogData` message.
    /// # Parameters
    /// - data: `LogData` message to take data from.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type appropraite for the `LogData` message,
    ///     - is one-dimentional,
    /// and should return an error otherwise.
    fn append_f144_value_slice(&self, data: &f144_LogData<'_>) -> NexusHDF5Result<()>;

    /// Implementation should append values from the given flatbuffer `SELog` message.
    /// # Parameters
    /// - data: `SELog` message to take data from.
    /// # Error Modes
    /// - The implementation should require that the dataset:
    ///     - was created with type appropraite for the `SELog` message,
    ///     - is one-dimentional,
    /// and should return an error otherwise.
    fn append_se00_value_slice(&self, data: &se00_SampleEnvironmentData<'_>)
        -> NexusHDF5Result<()>;
}

/// This trait provides methods to be called on the hdf5 `Attribute` type.
/// These methods provide additional guarantees that the resulting `Attribute`
/// is NeXus compliant.
pub(crate) trait AttributeExt {
    /// Implementation should set the value of the attribute to the given string slice value.
    /// # Parameters
    /// - value: slice to set the attribute to.
    /// # Error Modes
    /// - The implementation should require that the attribute:
    ///     - was created with type `hdf5::VarLenUnicode`,
    ///     - is scalar,
    /// and should return an error otherwise.
    fn set_string(&self, value: &str) -> NexusHDF5Result<()>;

    /// Implementation should return the timestamp from the attribute.
    /// # Return
    /// Implementation should attempt to return the selected [NexusDateTime].
    /// # Error Modes
    /// - The implementation should require that the attribute:
    ///     - was created with type `hdf5::VarLenUnicode`,
    ///     - is scalar,
    ///     - the contents can be parsed into a valid `NexusDateTime`,
    /// and should return an error if it was not.
    fn get_datetime(&self) -> NexusHDF5Result<NexusDateTime>;
    
    /// Implementation should return a [String] with the contents of the attribute.
    /// # Parameters
    /// - [String] with the contents of the attribute.
    /// # Return
    /// Implementation should attempt to return the selected string.
    /// # Error Modes
    /// - The implementation should require that the attribute:
    ///     - was created with type `hdf5::VarLenUnicode`,
    ///     - is scalar,
    /// and should return an error if it was not.
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
