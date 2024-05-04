use super::{add_new_group_to, create_resizable_dataset};
use crate::nexus::nexus_class as NX;
use anyhow::{anyhow, bail, Result};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, DatasetBuilderEmpty, Group, H5Type, SimpleExtents,
};
use ndarray::{s, Dim, SliceInfo, SliceInfoElem};
use std::{fmt::Debug, marker::PhantomData};
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::{f144_LogData, Value},
    ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion},
    flatbuffers::{Follow, Vector},
};

pub(super) type Slice1D = SliceInfo<[SliceInfoElem; 1], Dim<[usize; 1]>, Dim<[usize; 1]>>;

pub(super) trait TimeSeriesDataSource<'a>: Debug {
    fn write_values_to_dataset(&self, target: &Dataset) -> Result<usize>;
    fn get_hdf5_type(&self) -> Result<TypeDescriptor>;
}

impl<'a> TimeSeriesDataSource<'a> for f144_LogData<'a> {
    #[tracing::instrument(skip(self))]
    fn write_values_to_dataset(&self, target: &Dataset) -> Result<usize> {
        let type_descriptor = self.get_hdf5_type();
        let size = target.size();
        let slice = s![size..(size + 1)];
        let error = anyhow!("Could not convert value to type {type_descriptor:?}");
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => {
                    target.write_slice(&[self.value_as_byte().ok_or(error)?.value()], slice)
                }
                IntSize::U2 => {
                    target.write_slice(&[self.value_as_short().ok_or(error)?.value()], slice)
                }
                IntSize::U4 => {
                    target.write_slice(&[self.value_as_int().ok_or(error)?.value()], slice)
                }
                IntSize::U8 => {
                    target.write_slice(&[self.value_as_long().ok_or(error)?.value()], slice)
                }
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => {
                    target.write_slice(&[self.value_as_ubyte().ok_or(error)?.value()], slice)
                }
                IntSize::U2 => {
                    target.write_slice(&[self.value_as_ushort().ok_or(error)?.value()], slice)
                }
                IntSize::U4 => {
                    target.write_slice(&[self.value_as_uint().ok_or(error)?.value()], slice)
                }
                IntSize::U8 => {
                    target.write_slice(&[self.value_as_ulong().ok_or(error)?.value()], slice)
                }
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => {
                    target.write_slice(&[self.value_as_float().ok_or(error)?.value()], slice)
                }
                FloatSize::U8 => {
                    target.write_slice(&[self.value_as_double().ok_or(error)?.value()], slice)
                }
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }?;
        Ok(1)
    }

    fn get_hdf5_type(fb_type: Value) -> Result<TypeDescriptor> {
        let datatype = match fb_type {
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
            t => bail!(
                "Invalid flatbuffers logdata type {}",
                t.variant_name().unwrap()
            ),
        };
        Ok(datatype)
    }
}

fn write_generic_se_slice_to_dataset<'a, T: Follow<'a>>(
    vec: Vector<'a, T>,
    position: usize,
    target: Dataset,
) -> usize
where
    T::Inner: H5Type,
{
    let size = vec.len();
    let slice = s![position..(position + size)];
    target.write_slice(vec.iter().collect::<Vec<_>>(), slice);
    size
}

impl<'a> TimeSeriesDataSource<'a> for se00_SampleEnvironmentData<'a> {
    #[tracing::instrument(skip(self))]
    fn write_values_to_dataset(&self, target: &mut Dataset) -> Result<usize> {
        let type_descriptor = self.get_hdf5_type();
        let position = target.size();
        let error = anyhow!("Could not convert value to type {type_descriptor:?}");
        let size = match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => write_generic_se_slice_to_dataset(
                    &self.values_as_int_8_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                IntSize::U2 => write_generic_se_slice_to_dataset(
                    &self.values_as_int_16_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                IntSize::U4 => write_generic_se_slice_to_dataset(
                    &self.values_as_int_32_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                IntSize::U8 => write_generic_se_slice_to_dataset(
                    &self.values_as_int_64_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => write_generic_se_slice_to_dataset(
                    &self.values_as_uint_8_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                IntSize::U2 => write_generic_se_slice_to_dataset(
                    &self.values_as_uint_16_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                IntSize::U4 => write_generic_se_slice_to_dataset(
                    &self.values_as_uint_32_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                IntSize::U8 => write_generic_se_slice_to_dataset(
                    &self.values_as_uint_64_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => write_generic_se_slice_to_dataset(
                    &self.values_as_float_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
                FloatSize::U8 => write_generic_se_slice_to_dataset(
                    &self.values_as_double_array().ok_or(error)?.value(),
                    position,
                    target,
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }?;
        Ok(size)
    }

    fn get_hdf5_type(fb_type: ValueUnion) -> Result<TypeDescriptor> {
        let datatype = match fb_type {
            ValueUnion::Byte => TypeDescriptor::Integer(IntSize::U1),
            ValueUnion::UByte => TypeDescriptor::Unsigned(IntSize::U1),
            ValueUnion::Short => TypeDescriptor::Integer(IntSize::U2),
            ValueUnion::UShort => TypeDescriptor::Unsigned(IntSize::U2),
            ValueUnion::Int => TypeDescriptor::Integer(IntSize::U4),
            ValueUnion::UInt => TypeDescriptor::Unsigned(IntSize::U4),
            ValueUnion::Long => TypeDescriptor::Integer(IntSize::U8),
            ValueUnion::ULong => TypeDescriptor::Unsigned(IntSize::U8),
            ValueUnion::Float => TypeDescriptor::Float(FloatSize::U4),
            ValueUnion::Double => TypeDescriptor::Float(FloatSize::U8),
            t => bail!(
                "Invalid flatbuffers logdata type {}",
                t.variant_name().unwrap()
            ),
        };
        Ok(datatype)
    }
}

pub(super) trait TimeSeriesOwner<'a> {
    type DataSource;

    fn write_data_value_to_dataset_slice(
        type_descriptor: &TypeDescriptor,
        source: &Self::DataSource,
        target: &mut Dataset,
        slice: Slice1D,
    ) -> Result<()>;
}

#[derive(Debug)]
pub(crate) struct TimeSeries<OWNER> {
    timestamps: Dataset,
    values: Dataset,
    type_descriptor: TypeDescriptor,
    phantom: PhantomData<OWNER>,
}

impl<'a, OWNER: TimeSeriesOwner<'a>> TimeSeries<OWNER> {
    #[tracing::instrument(skip(parent))]
    pub(super) fn new_timeseries(
        parent: &Group,
        source_name: String,
        type_descriptor: TypeDescriptor,
    ) -> Result<Self> {
        let log = add_new_group_to(parent, &source_name, NX::RUNLOG)?;
        let timestamps = create_resizable_dataset::<i32>(&log, "time", 0, 1024)?;
        let values = get_dataset_builder(&log, &type_descriptor)?
            .shape(SimpleExtents::resizable(vec![0]))
            .chunk(1024)
            .create("value")?;

        Ok(TimeSeries::<OWNER> {
            timestamps,
            values,
            type_descriptor,
            phantom: Default::default(),
        })
    }

    #[tracing::instrument(skip(group))]
    pub(super) fn open_from_timeseries_group(group: &Group) -> Result<Self> {
        let timestamps = group.dataset("time")?;
        let values = group.dataset("value")?;
        let type_descriptor = values.dtype()?.to_descriptor()?;
        if !matches!(
            type_descriptor,
            TypeDescriptor::Integer(_) | TypeDescriptor::Unsigned(_) | TypeDescriptor::Float(_)
        ) {
            bail!("Invalid TypeDescriptor: {type_descriptor}");
        }

        Ok(TimeSeries::<OWNER> {
            timestamps,
            values,
            type_descriptor,
            phantom: Default::default(),
        })
    }

    #[tracing::instrument(skip(self))]
    pub(super) fn push_to_timeseries(&mut self, data: &OWNER::DataSource) -> Result<()> {
        Ok(())
    }
}

pub(super) fn get_dataset_builder(
    type_descriptor: &TypeDescriptor,
    parent: &Group,
) -> Result<DatasetBuilderEmpty> {
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
            return Err(anyhow!(
                "Invalid HDF5 array type: {}",
                type_descriptor.to_string()
            ))
        }
    })
}
