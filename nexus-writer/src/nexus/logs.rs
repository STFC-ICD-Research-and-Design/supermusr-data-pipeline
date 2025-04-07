use std::ops::Deref;

use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor, VarLenUnicode},
    Dataset,
};
use supermusr_streaming_types::{
    ecs_al00_alarm_generated::Alarm,
    ecs_f144_logdata_generated::{f144_LogData, Value},
    ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion},
};
use tracing::{trace, warn};

use crate::{
    error::{FlatBufferInvalidDataTypeContext, FlatBufferMissingError},
    hdf5_handlers::{
        ConvertResult, DatasetExt, DatasetFlatbuffersExt, NexusHDF5Error, NexusHDF5Result,
    },
    run_engine::{NexusDateTime, SampleEnvironmentLog},
};

pub(crate) struct LogWithOrigin<'a, T> {
    log: &'a T,
    origin: &'a NexusDateTime,
}

impl<'a, T> LogWithOrigin<'a, T> {
    pub(crate) fn get_origin(&self) -> &'a NexusDateTime {
        self.origin
    }
}

impl<T> Deref for LogWithOrigin<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.log
    }
}

pub(crate) trait LogMessage<'a>: Sized {
    fn get_name(&self) -> String;
    fn get_type_descriptor(&self) -> NexusHDF5Result<TypeDescriptor>;
    fn as_ref_with_origin(&'a self, origin: &'a NexusDateTime) -> LogWithOrigin<'a, Self> {
        LogWithOrigin { log: self, origin }
    }

    fn append_timestamps(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;
    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
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

impl<'a> LogMessage<'a> for f144_LogData<'a> {
    fn get_name(&self) -> String {
        remove_prefixes(self.source_name())
    }

    fn get_type_descriptor(&self) -> NexusHDF5Result<TypeDescriptor> {
        let error = |value: Value| {
            NexusHDF5Error::new_flatbuffer_invalid_data_type(
                FlatBufferInvalidDataTypeContext::RunLog,
                value
                    .variant_name()
                    .map(ToOwned::to_owned)
                    .unwrap_or_default(),
            )
        };
        let datatype = match self.value_type() {
            Value::Byte => TypeDescriptor::Integer(IntSize::U1),
            Value::UByte => TypeDescriptor::Unsigned(IntSize::U1),
            Value::Short => TypeDescriptor::Integer(IntSize::U2),
            Value::UShort => TypeDescriptor::Unsigned(IntSize::U2),
            Value::Int => TypeDescriptor::Integer(IntSize::U4),
            Value::UInt => TypeDescriptor::Unsigned(IntSize::U4),
            Value::Long => TypeDescriptor::Integer(IntSize::U8),
            Value::ULong => TypeDescriptor::Unsigned(IntSize::U8),
            Value::Float => TypeDescriptor::Float(FloatSize::U4),
            Value::Double => TypeDescriptor::Float(FloatSize::U8),
            value => return Err(error(value)),
        };
        Ok(datatype)
    }

    fn append_timestamps(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        dataset
            .append_value(adjust_nanoseconds_by_origin_to_sec(
                self.timestamp(),
                origin_time,
            ))
            .err_dataset(dataset)
    }

    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        dataset.append_f144_value_slice(self).err_dataset(dataset)
    }
}

fn get_se00_len(data: &se00_SampleEnvironmentData<'_>) -> NexusHDF5Result<usize> {
    let type_descriptor = data.get_type_descriptor()?;
    let error = || NexusHDF5Error::new_invalid_hdf5_type_conversion(type_descriptor.clone());
    match type_descriptor {
        TypeDescriptor::Integer(int_size) => match int_size {
            IntSize::U1 => data.values_as_int_8_array().map(|x| x.value().len()),
            IntSize::U2 => data.values_as_int_16_array().map(|x| x.value().len()),
            IntSize::U4 => data.values_as_int_32_array().map(|x| x.value().len()),
            IntSize::U8 => data.values_as_int_64_array().map(|x| x.value().len()),
        },
        TypeDescriptor::Unsigned(int_size) => match int_size {
            IntSize::U1 => data.values_as_uint_8_array().map(|x| x.value().len()),
            IntSize::U2 => data.values_as_uint_16_array().map(|x| x.value().len()),
            IntSize::U4 => data.values_as_uint_32_array().map(|x| x.value().len()),
            IntSize::U8 => data.values_as_uint_64_array().map(|x| x.value().len()),
        },
        TypeDescriptor::Float(float_size) => match float_size {
            FloatSize::U4 => data.values_as_float_array().map(|x| x.value().len()),
            FloatSize::U8 => data.values_as_double_array().map(|x| x.value().len()),
        },
        _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
    }
    .ok_or_else(error)
}

impl<'a> LogMessage<'a> for se00_SampleEnvironmentData<'a> {
    fn get_name(&self) -> String {
        remove_prefixes(self.name())
    }

    fn get_type_descriptor(&self) -> Result<TypeDescriptor, NexusHDF5Error> {
        let error = |t: ValueUnion| {
            NexusHDF5Error::new_flatbuffer_invalid_data_type(
                FlatBufferInvalidDataTypeContext::SELog,
                t.variant_name().map(ToOwned::to_owned).unwrap_or_default(),
            )
        };
        let datatype = match self.values_type() {
            ValueUnion::Int8Array => TypeDescriptor::Integer(IntSize::U1),
            ValueUnion::UInt8Array => TypeDescriptor::Unsigned(IntSize::U1),
            ValueUnion::Int16Array => TypeDescriptor::Integer(IntSize::U2),
            ValueUnion::UInt16Array => TypeDescriptor::Unsigned(IntSize::U2),
            ValueUnion::Int32Array => TypeDescriptor::Integer(IntSize::U4),
            ValueUnion::UInt32Array => TypeDescriptor::Unsigned(IntSize::U4),
            ValueUnion::Int64Array => TypeDescriptor::Integer(IntSize::U8),
            ValueUnion::UInt64Array => TypeDescriptor::Unsigned(IntSize::U8),
            ValueUnion::FloatArray => TypeDescriptor::Float(FloatSize::U4),
            ValueUnion::DoubleArray => TypeDescriptor::Float(FloatSize::U8),
            value_union => return Err(error(value_union)),
        };
        Ok(datatype)
    }

    fn append_timestamps(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        let num_values = get_se00_len(self).err_dataset(dataset)?;
        if let Some(timestamps) = self.timestamps() {
            let timestamps = timestamps
                .iter()
                .map(|t| adjust_nanoseconds_by_origin_to_sec(t, origin_time))
                .collect::<Vec<_>>();

            if timestamps.len() != num_values {
                return Err(NexusHDF5Error::FlatBufferInconsistentSELogTimeValueSizes {
                    sizes: (timestamps.len(), num_values),
                    hdf5_path: None,
                })
                .err_dataset(dataset);
            }
            dataset.append_slice(timestamps.as_slice())
        } else if self.time_delta() > 0.0 {
            trace!("Calculate times automatically.");

            let timestamps = (0..num_values)
                .map(|v| (v as f64 * self.time_delta()) as i64)
                .map(|t| {
                    adjust_nanoseconds_by_origin_to_sec(t + self.packet_timestamp(), origin_time)
                })
                .collect::<Vec<_>>();

            dataset.append_slice(timestamps.as_slice())
        } else {
            warn!("No time data.");
            Ok(())
        }
        .err_dataset(dataset)
    }

    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        dataset.append_se00_value_slice(self).err_dataset(dataset)
    }
}

impl<'a> LogMessage<'a> for SampleEnvironmentLog<'a> {
    fn get_name(&self) -> String {
        match self {
            SampleEnvironmentLog::LogData(data) => data.get_name(),
            SampleEnvironmentLog::SampleEnvironmentData(data) => data.get_name(),
        }
    }

    fn get_type_descriptor(&self) -> Result<TypeDescriptor, NexusHDF5Error> {
        match self {
            SampleEnvironmentLog::LogData(data) => data.get_type_descriptor(),
            SampleEnvironmentLog::SampleEnvironmentData(data) => data.get_type_descriptor(),
        }
    }

    fn append_timestamps(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        match self {
            SampleEnvironmentLog::LogData(data) => data.append_timestamps(dataset, origin_time),
            SampleEnvironmentLog::SampleEnvironmentData(data) => {
                data.append_timestamps(dataset, origin_time)
            }
        }
    }

    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        match self {
            SampleEnvironmentLog::LogData(data) => data.append_values(dataset),
            SampleEnvironmentLog::SampleEnvironmentData(data) => data.append_values(dataset),
        }
    }
}

pub(crate) trait AlarmMessage<'a>: Sized {
    fn as_ref_with_origin(&'a self, origin: &'a NexusDateTime) -> LogWithOrigin<'a, Self> {
        LogWithOrigin { log: self, origin }
    }

    fn get_name(&self) -> NexusHDF5Result<String>;

    fn append_timestamp(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;
    fn append_severity(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
    fn append_message(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
}

impl<'a> AlarmMessage<'a> for Alarm<'a> {
    fn get_name(&self) -> NexusHDF5Result<String> {
        let name = self
            .source_name()
            .ok_or_else(|| NexusHDF5Error::FlatBufferMissing {
                error: FlatBufferMissingError::AlarmName,
                hdf5_path: None,
            })?;
        Ok(remove_prefixes(name))
    }

    fn append_timestamp(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        dataset
            .append_value(adjust_nanoseconds_by_origin_to_sec(
                self.timestamp(),
                origin_time,
            ))
            .err_dataset(dataset)
    }

    fn append_severity(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        let severity = self
            .severity()
            .variant_name()
            .ok_or_else(|| {
                NexusHDF5Error::new_flatbuffer_missing(FlatBufferMissingError::AlarmSeverity)
            })
            .err_dataset(dataset)?;
        let severity = severity.parse::<VarLenUnicode>().err_dataset(dataset)?;
        dataset.append_value(severity).err_dataset(dataset)
    }

    fn append_message(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        let severity = self
            .message()
            .ok_or_else(|| {
                NexusHDF5Error::new_flatbuffer_missing(FlatBufferMissingError::AlarmMessage)
            })
            .err_dataset(dataset)?;
        let severity = severity.parse::<VarLenUnicode>().err_dataset(dataset)?;
        dataset.append_value(severity).err_dataset(dataset)
    }
}
