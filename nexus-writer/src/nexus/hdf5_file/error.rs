use std::{str::FromStr, string::ParseError};

use chrono::{DateTime, Utc};
use hdf5::{types::{TypeDescriptor, VarLenUnicode}, Attribute, Dataset, Group, Location};
use thiserror::Error;
use std::error::Error;

pub(crate) type NexusWriterResult<T> = Result<T, NexusWriterError>;

pub(crate) trait ConvertResult<T,E> where E : Error + Into<NexusWriterErrorType> {
    fn err_group(self, group : &Group) -> NexusWriterResult<T>;
    fn err_dataset(self, dataset : &Dataset) -> NexusWriterResult<T>;
    fn err_attribute(self, attribute : &Attribute) -> NexusWriterResult<T>;
}

#[derive(Debug,Error)]
#[error("{error_type} at {context}")]
pub(crate) struct NexusWriterError {
    error_type: NexusWriterErrorType,
    context: String,
}

impl<T,E> ConvertResult<T,E> for Result<T,E> where E : Error + Into<NexusWriterErrorType> {
    fn err_group(self, group : &Group) -> NexusWriterResult<T> {
        self.map_err(|e|
            NexusWriterError {
                error_type: e.into(),
                context: group.name(),
            })
    }

    fn err_dataset(self, dataset : &Dataset) -> NexusWriterResult<T> {
        self.map_err(|e|
            NexusWriterError {
                error_type: e.into(),
                context: dataset.name(),
            })
    }

    fn err_attribute(self, attribute : &Attribute) -> NexusWriterResult<T> {
        self.map_err(|e|
            NexusWriterError {
                error_type: e.into(),
                context: attribute.name(),
            })
    }
}

#[derive(Debug, Error)]
pub(crate) enum NexusWriterErrorType {
    #[error("{0}")]
    HDF5(#[from]hdf5::Error),
    //#[error("{0}")]
    //StringParse(#[from]ParseError),
    //#[error("{0}")]
    //VarLenUnicodeConversion(#[from]<VarLenUnicode as FromStr>::Err),
    #[error("{0}")]
    DateTimeConversion(#[from]chrono::ParseError),
    #[error("{0}")]
    HDF5String(#[from]hdf5::types::StringError),
    #[error("Flatbuffer Timestamp Missing")]
    FlatbufferMissingTimestamp,
    #[error("Flatbuffer Channels Missing")]
    FlatbufferMissingChannels,
    #[error("Flatbuffer Intensities Missing")]
    FlatbufferMissingIntensities,
    #[error("Flatbuffer Times Missing")]
    FlatbufferMissingTimes,
    #[error("Invalid HDF5 Type")]
    InvalidHDF5Type(TypeDescriptor),
}