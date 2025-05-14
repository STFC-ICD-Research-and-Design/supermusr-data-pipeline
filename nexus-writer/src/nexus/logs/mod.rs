//! This module defines the `LogMessage` and `AlarmMessage` traits which are implemented
//! for appropriate flatbuffer messages. They allow these messages to write their data
//! into a given Dataset.

mod alarm;
mod f114;
mod se00;

use crate::{hdf5_handlers::NexusHDF5Result, run_engine::NexusDateTime};
use hdf5::{types::TypeDescriptor, Dataset};

/// Helper trait that should be implemented on [supermusr_streaming_types::ecs_f144_logdata_generated::f144_LogData].
pub(crate) trait LogMessage<'a>: Sized {
    /// Implementation should return name of the log message.
    fn get_name(&self) -> String;

    /// Implementation should return data type of the log message.
    fn get_type_descriptor(&self) -> NexusHDF5Result<TypeDescriptor>;

    /// Implementation should append given dataset with the log message time values.
    /// # Parameters
    /// - dataset: [Dataset] to write data to.
    /// - origin_time: the time by which the timestamps should be written relative to. Usually the start time of the run.
    /// # Error Modes
    /// The implementation should require that the dataset:
    /// - was created with type appropraite for the `LogData` message,
    /// - is one-dimentional,
    ///
    /// and should return an error otherwise.
    fn append_timestamps_to(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;

    /// Implementation should append given dataset with the log message data values.
    /// # Parameters
    /// - dataset: [Dataset] to write data to.
    /// # Error Modes
    /// The implementation should require that the dataset:
    /// - was created with type appropraite for the `LogData` message,
    /// - is one-dimentional,
    ///
    /// and should return an error otherwise.
    fn append_values_to(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
}

pub(crate) trait AlarmMessage<'a>: Sized {
    fn get_name(&self) -> NexusHDF5Result<String>;

    /// # Parameters
    /// - dataset: [Dataset] to write data to.
    /// - origin_time: the time by which the timestamps should be written relative to. Usually the start time of the run.
    /// # Error Modes
    /// The implementation should require that the dataset:
    /// - was created with the [f64] type,
    /// - is one-dimentional,
    ///
    /// and should return an error otherwise.
    fn append_timestamp_to(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;

    /// # Parameters
    /// - dataset: [Dataset] to write data to.
    /// # Error Modes
    /// The implementation should require that the dataset:
    /// - was created with [hdf5::types::VarLenUnicode] type,
    /// - is one-dimentional,
    ///
    /// and should return an error otherwise.
    fn append_severity_to(&self, dataset: &Dataset) -> NexusHDF5Result<()>;

    /// # Parameters
    /// - dataset: [Dataset] to write data to.
    /// # Error Modes
    /// The implementation should require that the dataset:
    /// - was created with [hdf5::types::VarLenUnicode] type,
    /// - is one-dimentional,
    ///
    /// and should return an error otherwise.
    fn append_message_to(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
}

/// Coverts ns since epoch to ns since `origin_time`.
/// # Parameters
/// - nanoseconds: time since epoch to adjust.
/// - origin_time: timestamp to set the time relative to.
/// # Return
/// Time relative to the origin time in seconds.
fn adjust_nanoseconds_by_origin_to_sec(nanoseconds: i64, origin_time: &NexusDateTime) -> f64 {
    (origin_time
        .timestamp_nanos_opt()
        .map(|origin_time_ns| nanoseconds - origin_time_ns)
        .unwrap_or_default() as f64)
        / 1_000_000_000.0
}

/// Removes prefixes from sample environment log names.
/// # Parameters
/// - text: a string slice of the form: "prefix_1:prefix_2:...:prefix_n:LOG_NAME".
/// # Return
/// A string containing "LOG_NAME".
fn remove_prefixes(text: &str) -> String {
    text.split(':')
        .last()
        .expect("split contains at least one element, this should never fail")
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_prefixes() {
        assert_eq!(&remove_prefixes("one"), "one");
        assert_eq!(&remove_prefixes("one:two"), "two");
        assert_eq!(&remove_prefixes("one:two:three"), "three");

        assert_eq!(&remove_prefixes("one and a half"), "one and a half");
        assert_eq!(
            &remove_prefixes("one and a half:two and a half"),
            "two and a half"
        );
        assert_eq!(
            &remove_prefixes("one and a half:two and a half:three and a half"),
            "three and a half"
        );
    }
}
