mod alarm;
mod f114;
mod se00;

use crate::{hdf5_handlers::NexusHDF5Result, run_engine::NexusDateTime};
use hdf5::{types::TypeDescriptor, Dataset};

pub(crate) trait LogMessage<'a>: Sized {
    fn get_name(&self) -> String;
    fn get_type_descriptor(&self) -> NexusHDF5Result<TypeDescriptor>;

    fn append_timestamps_to(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;
    fn append_values_to(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
}

pub(crate) trait AlarmMessage<'a>: Sized {
    fn get_name(&self) -> NexusHDF5Result<String>;

    fn append_timestamp_to(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;
    fn append_severity_to(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
    fn append_message_to(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
}

fn adjust_nanoseconds_by_origin_to_sec(nanoseconds: i64, origin_time: &NexusDateTime) -> f64 {
    (origin_time
        .timestamp_nanos_opt()
        .map(|origin_time_ns| nanoseconds - origin_time_ns)
        .unwrap_or_default() as f64)
        / 1_000_000_000.0
}

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
