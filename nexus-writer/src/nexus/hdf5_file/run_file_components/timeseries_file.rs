use chrono::{DateTime, Utc};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, DatasetBuilderEmpty, Group, H5Type,
};
use ndarray::{s, Dim, SliceInfo, SliceInfoElem};
use std::fmt::Debug;
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::{f144_LogData, Value},
    ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion},
    flatbuffers::{Follow, Vector},
};
use tracing::{debug, trace};

use crate::nexus::hdf5_file::hdf5_writer::add_attribute_to;

pub(super) type Slice1D = SliceInfo<[SliceInfoElem; 1], Dim<[usize; 1]>, Dim<[usize; 1]>>;

pub(super) trait TimeSeriesDataSource<'a>: Debug {
    fn write_initial_timestamp(&self, target: &Dataset) -> anyhow::Result<()>;
    fn write_timestamps_to_dataset(
        &self,
        target: &Dataset,
        num_values: usize,
    ) -> anyhow::Result<()>;
    fn write_values_to_dataset(&self, target: &Dataset) -> anyhow::Result<usize>;
    fn get_hdf5_type(&self) -> anyhow::Result<TypeDescriptor>;
}

pub(super) fn get_dataset_builder(
    type_descriptor: &TypeDescriptor,
    parent: &Group,
) -> anyhow::Result<DatasetBuilderEmpty> {
    Ok(match type_descriptor {
        TypeDescriptor::Integer(sz) => match sz {
            IntSize::U1 => parent.new_dataset::<i8>(),
            IntSize::U2 => parent.new_dataset::<i16>(),
            IntSize::U4 => parent.new_dataset::<i32>(),
            IntSize::U8 => parent.new_dataset::<i64>(),
        },
        TypeDescriptor::Unsigned(sz) => match sz {
            IntSize::U1 => parent.new_dataset::<u8>(),
            IntSize::U2 => parent.new_dataset::<u16>(),
            IntSize::U4 => parent.new_dataset::<u32>(),
            IntSize::U8 => parent.new_dataset::<u64>(),
        },
        TypeDescriptor::Float(sz) => match sz {
            FloatSize::U4 => parent.new_dataset::<f32>(),
            FloatSize::U8 => parent.new_dataset::<f64>(),
        },
        _ => {
            return Err(anyhow::anyhow!(
                "Invalid HDF5 array type: {}",
                type_descriptor.to_string()
            ))
        }
    })
}

fn write_generic_logdata_slice_to_dataset<T: H5Type>(
    val: T,
    target: &Dataset,
) -> anyhow::Result<Slice1D> {
    let position = target.size();
    let slice = s![position..(position + 1)];
    target.resize(position + 1)?;
    target.write_slice(&[val], slice)?;
    Ok(slice)
}

impl<'a> TimeSeriesDataSource<'a> for f144_LogData<'a> {
    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_initial_timestamp(&self, target: &Dataset) -> anyhow::Result<()> {
        let time = DateTime::<Utc>::from_timestamp_nanos(self.timestamp()).to_rfc3339();
        add_attribute_to(target, "Start", &time)?;
        add_attribute_to(target, "Units", "second")?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_timestamps_to_dataset(
        &self,
        target: &Dataset,
        _num_values: usize,
    ) -> anyhow::Result<()> {
        let position = target.size();
        target.resize(position + 1)?;
        let slice = s![position..(position + 1)];
        debug!("Timestamp Slice: {slice:?}, Value: {0:?}", self.timestamp());
        target.write_slice(&[self.timestamp()], slice)?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_values_to_dataset(&self, target: &Dataset) -> anyhow::Result<usize> {
        let type_descriptor = self.get_hdf5_type()?;
        let error = anyhow::anyhow!("Could not convert value to type {type_descriptor:?}");
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => write_generic_logdata_slice_to_dataset(
                    self.value_as_byte().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U2 => write_generic_logdata_slice_to_dataset(
                    self.value_as_short().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U4 => write_generic_logdata_slice_to_dataset(
                    self.value_as_int().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U8 => write_generic_logdata_slice_to_dataset(
                    self.value_as_long().ok_or(error)?.value(),
                    target,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => write_generic_logdata_slice_to_dataset(
                    self.value_as_ubyte().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U2 => write_generic_logdata_slice_to_dataset(
                    self.value_as_ushort().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U4 => write_generic_logdata_slice_to_dataset(
                    self.value_as_uint().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U8 => write_generic_logdata_slice_to_dataset(
                    self.value_as_ulong().ok_or(error)?.value(),
                    target,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => write_generic_logdata_slice_to_dataset(
                    self.value_as_float().ok_or(error)?.value(),
                    target,
                ),
                FloatSize::U8 => write_generic_logdata_slice_to_dataset(
                    self.value_as_double().ok_or(error)?.value(),
                    target,
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }?;
        Ok(1)
    }

    fn get_hdf5_type(&self) -> anyhow::Result<TypeDescriptor> {
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
            t => anyhow::bail!("Invalid flatbuffers logdata type {:?}", t.variant_name()),
        };
        Ok(datatype)
    }
}

fn write_generic_se_slice_to_dataset<'a, T: Follow<'a>>(
    vec: Vector<'a, T>,
    target: &Dataset,
) -> anyhow::Result<usize>
where
    T::Inner: H5Type,
{
    let size = vec.len();
    let position = target.size();
    let slice = s![position..(position + size)];
    target.resize(position + size)?;
    target.write_slice(&vec.iter().collect::<Vec<_>>(), slice)?;
    Ok(size)
}

impl<'a> TimeSeriesDataSource<'a> for se00_SampleEnvironmentData<'a> {
    #[tracing::instrument(skip_all, err(level = "warn"))]
    fn write_initial_timestamp(&self, target: &Dataset) -> anyhow::Result<()> {
        let time = DateTime::<Utc>::from_timestamp_nanos(self.packet_timestamp()).to_rfc3339();
        add_attribute_to(target, "Start", &time)?;
        add_attribute_to(target, "Units", "second")?;
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_timestamps_to_dataset(
        &self,
        target: &Dataset,
        num_values: usize,
    ) -> anyhow::Result<()> {
        let position = target.size();
        if let Some(timestamps) = self.timestamps() {
            trace!("Times given explicitly.");

            let timestamps = timestamps.iter().collect::<Vec<_>>();
            if timestamps.len() != num_values {
                anyhow::bail!("Different number of values and times");
            }
            let slice = s![position..(position + num_values)];

            debug!("Timestamp Slice: {slice:?}, Times: {0:?}", timestamps);
            target.resize(position + num_values)?;
            target.write_slice(timestamps.as_slice(), slice)?;
        } else if self.time_delta() > 0.0 {
            trace!("Calculate times automatically.");

            let timestamps = (0..num_values)
                .map(|v| v as f64 * self.time_delta())
                .collect::<Vec<_>>();
            let slice = s![position..(position + num_values)];

            debug!("Timestamp Slice: {slice:?}, Times: {0:?}", timestamps,);
            target.write_slice(timestamps.as_slice(), slice)?;
        } else {
            trace!("No time data.");
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "trace", err(level = "warn"))]
    fn write_values_to_dataset(&self, target: &Dataset) -> anyhow::Result<usize> {
        let type_descriptor = self.get_hdf5_type()?;
        let error = anyhow::anyhow!("Could not convert value to type {type_descriptor:?}");
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => write_generic_se_slice_to_dataset(
                    self.values_as_int_8_array().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U2 => write_generic_se_slice_to_dataset(
                    self.values_as_int_16_array().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U4 => write_generic_se_slice_to_dataset(
                    self.values_as_int_32_array().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U8 => write_generic_se_slice_to_dataset(
                    self.values_as_int_64_array().ok_or(error)?.value(),
                    target,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_8_array().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U2 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_16_array().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U4 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_32_array().ok_or(error)?.value(),
                    target,
                ),
                IntSize::U8 => write_generic_se_slice_to_dataset(
                    self.values_as_uint_64_array().ok_or(error)?.value(),
                    target,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => write_generic_se_slice_to_dataset(
                    self.values_as_float_array().ok_or(error)?.value(),
                    target,
                ),
                FloatSize::U8 => write_generic_se_slice_to_dataset(
                    self.values_as_double_array().ok_or(error)?.value(),
                    target,
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }
    }

    fn get_hdf5_type(&self) -> anyhow::Result<TypeDescriptor> {
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
            t => anyhow::bail!("Invalid flatbuffers logdata type {:?}", t.variant_name()),
        };
        Ok(datatype)
    }
}
