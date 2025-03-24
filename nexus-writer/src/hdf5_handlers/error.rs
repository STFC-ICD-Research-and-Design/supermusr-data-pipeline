use crate::error::{FlatBufferInvalidDataTypeContext, FlatBufferMissingError};
use hdf5::{types::TypeDescriptor, Attribute, Dataset, Group};
use std::{error::Error, num::ParseIntError};
use supermusr_streaming_types::time_conversions::GpsTimeConversionError;
use thiserror::Error;

pub(crate) type NexusHDF5Result<T> = Result<T, NexusHDF5Error>;

const NO_HDF5_PATH_SET: &str = "[No HDF5 Path Set]";

#[derive(Debug, Error)]
pub(crate) enum NexusHDF5Error {
    #[error("{error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    HDF5 {
        error: hdf5::Error,
        hdf5_path: Option<String>,
    },
    #[error("{error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    DateTimeConversion {
        error: chrono::ParseError,
        hdf5_path: Option<String>,
    },
    #[error("{error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    HDF5String {
        error: hdf5::types::StringError,
        hdf5_path: Option<String>,
    },
    #[error("Flatbuffer Timestamp Conversion Error {error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    FlatBufferTimestampConversion {
        error: GpsTimeConversionError,
        hdf5_path: Option<String>,
    },
    #[error("Flatbuffer Timestamp Error Converting to Nanoseconds at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    FlatBufferTimestampConvertToNanoseconds { hdf5_path: Option<String> },
    #[error("{error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    FlatBufferMissing {
        error: FlatBufferMissingError,
        hdf5_path: Option<String>,
    },
    #[error("Invalid FlatBuffer {context} Data Type {error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    FlatBufferInvalidDataType {
        context: FlatBufferInvalidDataTypeContext,
        error: String,
        hdf5_path: Option<String>,
    },
    #[error("Inconsistent Numbers of Sample Environment Log Times and Values {0} != {1} at {2}", sizes.0, sizes.1, hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    FlatBufferInconsistentSELogTimeValueSizes {
        sizes: (usize, usize),
        hdf5_path: Option<String>,
    },
    #[error("Invalid HDF5 Type {error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    InvalidHDF5Type {
        error: TypeDescriptor,
        hdf5_path: Option<String>,
    },
    #[error("Invalid HDF5 Conversion {error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    InvalidHDF5TypeConversion {
        error: TypeDescriptor,
        hdf5_path: Option<String>,
    },
    #[error("IO Error {error} at {0}", hdf5_path.as_deref().unwrap_or(NO_HDF5_PATH_SET))]
    IO {
        error: std::io::Error,
        hdf5_path: Option<String>,
    },
    #[error("Integer Conversion From String Error")]
    ParseInt{
        error: std::num::ParseIntError,
        hdf5_path: Option<String>,
    },
}

impl NexusHDF5Error {
    fn with_hdf5_path(self, path: String) -> Self {
        match self {
            Self::HDF5 {
                error,
                hdf5_path: _,
            } => Self::HDF5 {
                error,
                hdf5_path: Some(path),
            },
            Self::DateTimeConversion {
                error,
                hdf5_path: _,
            } => Self::DateTimeConversion {
                error,
                hdf5_path: Some(path),
            },
            Self::HDF5String {
                error,
                hdf5_path: _,
            } => Self::HDF5String {
                error,
                hdf5_path: Some(path),
            },
            Self::FlatBufferTimestampConversion {
                error,
                hdf5_path: _,
            } => Self::FlatBufferTimestampConversion {
                error,
                hdf5_path: Some(path),
            },
            Self::FlatBufferTimestampConvertToNanoseconds { hdf5_path: _ } => {
                Self::FlatBufferTimestampConvertToNanoseconds {
                    hdf5_path: Some(path),
                }
            }
            Self::FlatBufferMissing {
                error,
                hdf5_path: _,
            } => Self::FlatBufferMissing {
                error,
                hdf5_path: Some(path),
            },
            Self::FlatBufferInvalidDataType {
                context,
                error,
                hdf5_path: _,
            } => Self::FlatBufferInvalidDataType {
                context,
                error,
                hdf5_path: Some(path),
            },
            Self::FlatBufferInconsistentSELogTimeValueSizes {
                sizes,
                hdf5_path: _,
            } => Self::FlatBufferInconsistentSELogTimeValueSizes {
                sizes,
                hdf5_path: Some(path),
            },
            Self::InvalidHDF5Type {
                error,
                hdf5_path: _,
            } => Self::InvalidHDF5Type {
                error,
                hdf5_path: Some(path),
            },
            Self::InvalidHDF5TypeConversion {
                error,
                hdf5_path: _,
            } => Self::InvalidHDF5TypeConversion {
                error,
                hdf5_path: Some(path),
            },
            Self::IO {
                error,
                hdf5_path: _,
            } => Self::IO {
                error,
                hdf5_path: Some(path),
            },
            Self::ParseInt {
                error,
                hdf5_path: _,
            } => Self::ParseInt {
                error,
                hdf5_path: Some(path),
            },
        }
    }

    pub(crate) fn new_flatbuffer_missing(error: FlatBufferMissingError) -> Self {
        Self::FlatBufferMissing {
            error,
            hdf5_path: None,
        }
    }

    pub(crate) fn new_flatbuffer_timestamp_convert_to_nanoseconds() -> Self {
        Self::FlatBufferTimestampConvertToNanoseconds { hdf5_path: None }
    }

    pub(crate) fn new_flatbuffer_invalid_data_type(
        context: FlatBufferInvalidDataTypeContext,
        error: String,
    ) -> Self {
        Self::FlatBufferInvalidDataType {
            context,
            error,
            hdf5_path: None,
        }
    }

    pub(crate) fn new_invalid_hdf5_type_conversion(error: TypeDescriptor) -> Self {
        Self::InvalidHDF5TypeConversion {
            error,
            hdf5_path: None,
        }
    }
}

impl From<std::num::ParseIntError> for NexusHDF5Error {
    fn from(error: std::num::ParseIntError) -> Self {
        NexusHDF5Error::ParseInt {
            error,
            hdf5_path: None,
        }
    }
}

impl From<hdf5::Error> for NexusHDF5Error {
    fn from(error: hdf5::Error) -> Self {
        NexusHDF5Error::HDF5 {
            error,
            hdf5_path: None,
        }
    }
}

impl From<hdf5::types::StringError> for NexusHDF5Error {
    fn from(error: hdf5::types::StringError) -> Self {
        NexusHDF5Error::HDF5String {
            error,
            hdf5_path: None,
        }
    }
}

impl From<chrono::ParseError> for NexusHDF5Error {
    fn from(error: chrono::ParseError) -> Self {
        NexusHDF5Error::DateTimeConversion {
            error,
            hdf5_path: None,
        }
    }
}

impl From<GpsTimeConversionError> for NexusHDF5Error {
    fn from(error: GpsTimeConversionError) -> Self {
        NexusHDF5Error::FlatBufferTimestampConversion {
            error,
            hdf5_path: None,
        }
    }
}

impl From<FlatBufferMissingError> for NexusHDF5Error {
    fn from(error: FlatBufferMissingError) -> Self {
        NexusHDF5Error::FlatBufferMissing {
            error,
            hdf5_path: None,
        }
    }
}

impl From<std::io::Error> for NexusHDF5Error {
    fn from(error: std::io::Error) -> Self {
        NexusHDF5Error::IO {
            error,
            hdf5_path: None,
        }
    }
}

/// Used to allow errors which can be convertex to NexusHDF5Errors to be
/// appended with hdf5 paths
pub(crate) trait ConvertResult<T, E>
where
    E: Error + Into<NexusHDF5Error>,
{
    fn err_group(self, group: &Group) -> NexusHDF5Result<T>;
    fn err_dataset(self, dataset: &Dataset) -> NexusHDF5Result<T>;
    fn err_attribute(self, attribute: &Attribute) -> NexusHDF5Result<T>;
    fn err_file(self) -> NexusHDF5Result<T>;
}

impl<T, E> ConvertResult<T, E> for Result<T, E>
where
    E: Error + Into<NexusHDF5Error>,
{
    fn err_group(self, group: &Group) -> NexusHDF5Result<T> {
        self.map_err(|e| e.into().with_hdf5_path(group.name()))
    }

    fn err_dataset(self, dataset: &Dataset) -> NexusHDF5Result<T> {
        self.map_err(|e| e.into().with_hdf5_path(dataset.name()))
    }

    fn err_attribute(self, attribute: &Attribute) -> NexusHDF5Result<T> {
        self.map_err(|e| e.into().with_hdf5_path(attribute.name()))
    }

    fn err_file(self) -> NexusHDF5Result<T> {
        self.map_err(|e| e.into().with_hdf5_path("File Level".to_owned()))
    }
}
