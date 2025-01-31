use crate::nexus::{
    hdf5_file::{
        error::{ConvertResult, NexusHDF5ErrorType, NexusHDF5Result},
        hdf5_writer::DatasetExt,
    },
    NexusDateTime,
};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, H5Type,
};
use ndarray::{s, Dim, SliceInfo, SliceInfoElem};
use std::fmt::Debug;
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::{f144_LogData, Value},
    ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion},
    flatbuffers::{Follow, Vector},
};
use tracing::{debug, trace, warn};

pub(super) type Slice1D = SliceInfo<[SliceInfoElem; 1], Dim<[usize; 1]>, Dim<[usize; 1]>>;

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

pub(super) trait TimeSeriesDataSource<'a>: Debug {
    fn write_timestamps_to_dataset(
        &self,
        target: &Dataset,
        num_values: usize,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()>;
    fn write_values_to_dataset(&self, target: &Dataset) -> NexusHDF5Result<usize>;
    fn get_hdf5_type(&self) -> Result<TypeDescriptor, NexusHDF5ErrorType>;
}

fn write_generic_logdata_slice_to_dataset<T: H5Type>(
    val: T,
    target: &Dataset,
) -> NexusHDF5Result<Slice1D> {
    let position = target.size();
    let slice = s![position..(position + 1)];
    target.append_slice(&[val])?;
    Ok(slice)
}

impl<'a> TimeSeriesDataSource<'a> for f144_LogData<'a> {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_timestamps_to_dataset(
        &self,
        target: &Dataset,
        _num_values: usize,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        let position = target.size();
        let slice = s![position..(position + 1)];
        debug!("Timestamp Slice: {slice:?}, Value: {0:?}", self.timestamp());
        target.append_slice(&[adjust_nanoseconds_by_origin_to_sec(
            self.timestamp(),
            origin_time,
        )])?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_values_to_dataset(&self, target: &Dataset) -> NexusHDF5Result<usize> {
        let type_descriptor = self.get_hdf5_type().err_dataset(target)?;
        let error = NexusHDF5ErrorType::InvalidHDF5TypeConversion(type_descriptor.clone());
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => write_generic_logdata_slice_to_dataset(
                    self.value_as_byte()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U2 => write_generic_logdata_slice_to_dataset(
                    self.value_as_short()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U4 => write_generic_logdata_slice_to_dataset(
                    self.value_as_int()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U8 => write_generic_logdata_slice_to_dataset(
                    self.value_as_long()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => write_generic_logdata_slice_to_dataset(
                    self.value_as_ubyte()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U2 => write_generic_logdata_slice_to_dataset(
                    self.value_as_ushort()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U4 => write_generic_logdata_slice_to_dataset(
                    self.value_as_uint()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U8 => write_generic_logdata_slice_to_dataset(
                    self.value_as_ulong()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => write_generic_logdata_slice_to_dataset(
                    self.value_as_float()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                FloatSize::U8 => write_generic_logdata_slice_to_dataset(
                    self.value_as_double()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }?;
        Ok(1)
    }

    fn get_hdf5_type(&self) -> Result<TypeDescriptor, NexusHDF5ErrorType> {
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
            t => {
                return Err(NexusHDF5ErrorType::FlatBufferInvalidRunLogDataType(
                    t.variant_name().map(ToOwned::to_owned).unwrap_or_default(),
                ))
            }
        };
        Ok(datatype)
    }
}

fn write_generic_se_slice_to_dataset<'a, T: Follow<'a>>(
    vec: Vector<'a, T>,
    target: &Dataset,
) -> NexusHDF5Result<usize>
where
    T::Inner: H5Type,
{
    let size = vec.len();
    target.append_slice(&vec.iter().collect::<Vec<_>>())?;
    Ok(size)
}

impl<'a> TimeSeriesDataSource<'a> for se00_SampleEnvironmentData<'a> {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_timestamps_to_dataset(
        &self,
        target: &Dataset,
        num_values: usize,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        let position = target.size();
        if let Some(timestamps) = self.timestamps() {
            trace!("Times given explicitly.");

            let timestamps = timestamps
                .iter()
                .map(|t| adjust_nanoseconds_by_origin_to_sec(t, origin_time))
                .collect::<Vec<_>>();

            if timestamps.len() != num_values {
                return Err(
                    NexusHDF5ErrorType::FlatBufferInconsistentSELogTimeValueSizes(
                        timestamps.len(),
                        num_values,
                    ),
                )
                .err_dataset(target);
            }
            let slice = s![position..(position + num_values)];

            debug!("Timestamp Slice: {slice:?}, Times: {0:?}", timestamps);
            target.append_slice(timestamps.as_slice())?;
        } else if self.time_delta() > 0.0 {
            trace!("Calculate times automatically.");

            let timestamps = (0..num_values)
                .map(|v| (v as f64 * self.time_delta()) as i64)
                .map(|t| {
                    adjust_nanoseconds_by_origin_to_sec(t + self.packet_timestamp(), origin_time)
                })
                .collect::<Vec<_>>();
            let slice = s![position..(position + num_values)];

            debug!("Timestamp Slice: {slice:?}, Times: {0:?}", timestamps,);
            target.append_slice(timestamps.as_slice())?;
        } else {
            warn!("No time data.");
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_values_to_dataset(&self, target: &Dataset) -> NexusHDF5Result<usize> {
        let type_descriptor = self.get_hdf5_type().err_dataset(target)?;
        let error = NexusHDF5ErrorType::InvalidHDF5TypeConversion(type_descriptor.clone());
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => write_generic_se_slice_to_dataset(
                    self.values_as_int_8_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U2 => write_generic_se_slice_to_dataset(
                    self.values_as_int_16_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U4 => write_generic_se_slice_to_dataset(
                    self.values_as_int_32_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U8 => write_generic_se_slice_to_dataset(
                    self.values_as_int_64_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_8_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U2 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_16_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U4 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_32_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                IntSize::U8 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_64_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => write_generic_se_slice_to_dataset(
                    self.values_as_float_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
                FloatSize::U8 => write_generic_se_slice_to_dataset(
                    self.values_as_double_array()
                        .ok_or(error)
                        .err_dataset(target)?
                        .value(),
                    target,
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }
    }

    fn get_hdf5_type(&self) -> Result<TypeDescriptor, NexusHDF5ErrorType> {
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
            t => {
                return Err(NexusHDF5ErrorType::FlatBufferInvalidSELogDataType(
                    t.variant_name().map(ToOwned::to_owned).unwrap_or_default(),
                ))
            }
        };
        Ok(datatype)
    }
}
