use crate::nexus::error::FlatBufferMissingError;
use hdf5::{types::TypeDescriptor, Attribute, Dataset, Group};
use std::error::Error;
use supermusr_streaming_types::time_conversions::GpsTimeConversionError;
use thiserror::Error;

pub(crate) type NexusHDF5Result<T> = Result<T, NexusHDF5Error>;

pub(crate) trait ConvertResult<T, E>
where
    E: Error + Into<NexusHDF5ErrorType>,
{
    fn err_group(self, group: &Group) -> NexusHDF5Result<T>;
    fn err_dataset(self, dataset: &Dataset) -> NexusHDF5Result<T>;
    fn err_attribute(self, attribute: &Attribute) -> NexusHDF5Result<T>;
    fn err_file(self) -> NexusHDF5Result<T>;
}

#[derive(Debug, Error)]
#[error("{error_type} at {context}")]
pub(crate) struct NexusHDF5Error {
    error_type: NexusHDF5ErrorType,
    context: String,
}

impl<T, E> ConvertResult<T, E> for Result<T, E>
where
    E: Error + Into<NexusHDF5ErrorType>,
{
    fn err_group(self, group: &Group) -> NexusHDF5Result<T> {
        self.map_err(|e| NexusHDF5Error {
            error_type: e.into(),
            context: group.name(),
        })
    }

    fn err_dataset(self, dataset: &Dataset) -> NexusHDF5Result<T> {
        self.map_err(|e| NexusHDF5Error {
            error_type: e.into(),
            context: dataset.name(),
        })
    }

    fn err_attribute(self, attribute: &Attribute) -> NexusHDF5Result<T> {
        self.map_err(|e| NexusHDF5Error {
            error_type: e.into(),
            context: attribute.name(),
        })
    }

    fn err_file(self) -> NexusHDF5Result<T> {
        self.map_err(|e| NexusHDF5Error {
            error_type: e.into(),
            context: "File Level".to_owned(),
        })
    }
}

#[derive(Debug, Error)]
pub(crate) enum NexusHDF5ErrorType {
    #[error("{0}")]
    HDF5(#[from] hdf5::Error),
    #[error("{0}")]
    DateTimeConversion(#[from] chrono::ParseError),
    #[error("{0}")]
    HDF5String(#[from] hdf5::types::StringError),
    #[error("Flatbuffer Timestamp Conversion Error {0}")]
    FlatBufferTimestampConversion(#[from] GpsTimeConversionError),
    #[error("Flatbuffer Timestamp Calculation Error")]
    FlatBufferTimestampCalculation,
    #[error("{0}")]
    FlatBufferMissing(FlatBufferMissingError),
    #[error("Invalid FlatBuffer RunLog Data Type {0}")]
    FlatBufferInvalidRunLogDataType(String),
    #[error("Invalid FlatBuffer Sample Environment Log Data Type {0}")]
    FlatBufferInvalidSELogDataType(String),
    #[error("Inconsistent Numbers of SELog Times and Values {0} != {1}")]
    FlatBufferInconsistentSELogTimeValueSizes(usize, usize),
    #[error("Invalid HDF5 Type {0}")]
    InvalidHDF5Type(TypeDescriptor),
    #[error("Invalid HDF5 Conversion {0}")]
    InvalidHDF5TypeConversion(TypeDescriptor),
    #[error("IO Error {0}")]
    IO(#[from] std::io::Error),
}
