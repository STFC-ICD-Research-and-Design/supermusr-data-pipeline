use hdf5::{
    types::{self, FloatSize, IntSize, TypeDescriptor},
    Dataset,
};
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::{f144_LogData, Value},
    ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion},
    flatbuffers::Follow,
};
use tracing::{trace, warn};

use crate::{
    error::FlatBufferInvalidDataTypeContext,
    hdf5_handlers::{ConvertResult, NexusHDF5Error, NexusHDF5Result},
    run_engine::{DatasetExt, NexusDateTime, SampleEnvironmentLog},
};

pub(crate) trait LogMessage<'a> {
    fn get_name(&self) -> &'a str;
    fn get_type_descriptor(&self) -> NexusHDF5Result<TypeDescriptor>;

    fn append_timestamps(&self, dataset: &Dataset, origin_time: &NexusDateTime) -> NexusHDF5Result<()>;
    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()>;
}

impl<'a> LogMessage<'a> for f144_LogData<'a> {
    fn get_name(&self) -> &'a str {
        self.source_name()
    }

    fn get_type_descriptor(&self) -> Result<TypeDescriptor, NexusHDF5Error> {
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

    fn append_timestamps(&self, dataset: &Dataset, origin_time: &NexusDateTime) -> NexusHDF5Result<()> {
        dataset
            .append_slice(&[self.timestamp()])
            .err_dataset(dataset)
    }

    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        let type_descriptor = self.get_type_descriptor().err_dataset(dataset)?;
        let error = || NexusHDF5Error::new_invalid_hdf5_type_conversion(type_descriptor.clone());
        match type_descriptor {
            TypeDescriptor::Integer(int_size) => match int_size {
                IntSize::U1 => {
                    dataset.append_slice(&[self.value_as_byte().ok_or_else(error)?.value()])
                }
                IntSize::U2 => {
                    dataset.append_slice(&[self.value_as_short().ok_or_else(error)?.value()])
                }
                IntSize::U4 => {
                    dataset.append_slice(&[self.value_as_int().ok_or_else(error)?.value()])
                }
                IntSize::U8 => {
                    dataset.append_slice(&[self.value_as_long().ok_or_else(error)?.value()])
                }
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => {
                    dataset.append_slice(&[self.value_as_ubyte().ok_or_else(error)?.value()])
                }
                IntSize::U2 => {
                    dataset.append_slice(&[self.value_as_ushort().ok_or_else(error)?.value()])
                }
                IntSize::U4 => {
                    dataset.append_slice(&[self.value_as_uint().ok_or_else(error)?.value()])
                }
                IntSize::U8 => {
                    dataset.append_slice(&[self.value_as_ulong().ok_or_else(error)?.value()])
                }
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => {
                    dataset.append_slice(&[self.value_as_float().ok_or_else(error)?.value()])
                }
                FloatSize::U8 => {
                    dataset.append_slice(&[self.value_as_double().ok_or_else(error)?.value()])
                }
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
        }
        .err_dataset(dataset)
    }
}

pub(super) fn adjust_nanoseconds_by_origin_to_sec(
    nanoseconds: i64,
    origin_time: &NexusDateTime,
) -> f64 {
    (origin_time
        .timestamp_nanos_opt()
        .map(|origin_time_ns| nanoseconds - origin_time_ns)
        .unwrap_or_default() as f64)
        / 1_000_000_000.0
}

impl<'a> LogMessage<'a> for se00_SampleEnvironmentData<'a> {
    fn get_name(&self) -> &'a str {
        self.name()
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

    fn append_timestamps(&self, dataset: &Dataset, origin_time: &NexusDateTime) -> NexusHDF5Result<()> {
        let num_values = self.
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

    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<usize> {
        let type_descriptor = self.get_type_descriptor().err_dataset(dataset)?;
        let error = || NexusHDF5Error::new_invalid_hdf5_type_conversion(type_descriptor.clone());
        match type_descriptor {
            TypeDescriptor::Integer(int_size) => match int_size {
                IntSize::U1 => dataset.append_slice(
                    &self
                        .values_as_int_8_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U2 => dataset.append_slice(
                    &self
                        .values_as_int_16_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U4 => dataset.append_slice(
                    &self
                        .values_as_int_32_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U8 => dataset.append_slice(
                    &self
                        .values_as_int_64_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => dataset.append_slice(
                    &self
                        .values_as_uint_8_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U2 => dataset.append_slice(
                    &self
                        .values_as_uint_16_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U4 => dataset.append_slice(
                    &self
                        .values_as_uint_32_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U8 => dataset.append_slice(
                    &self
                        .values_as_uint_64_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => dataset.append_slice(
                    &self
                        .values_as_float_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                FloatSize::U8 => dataset.append_slice(
                    &self
                        .values_as_double_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
        }
        .err_dataset(dataset)
    }
}

impl<'a> LogMessage<'a> for SampleEnvironmentLog<'a> {
    fn get_name(&self) -> &'a str {
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

    fn append_timestamps(&self, dataset: &Dataset, num_values: usize, origin_time: &NexusDateTime) -> NexusHDF5Result<()> {
        match self {
            SampleEnvironmentLog::LogData(data) => data.append_timestamps(dataset, num_values, origin_time),
            SampleEnvironmentLog::SampleEnvironmentData(data) => data.append_timestamps(dataset, num_values, origin_time),
        }
    }

    fn append_values(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        match self {
            SampleEnvironmentLog::LogData(data) => data.append_values(dataset),
            SampleEnvironmentLog::SampleEnvironmentData(data) => data.append_values(dataset),
        }
    }
}
