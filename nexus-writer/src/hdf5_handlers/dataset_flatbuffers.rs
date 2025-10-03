//! This module implements the traits to extend the hdf5 [Dataset] type to provide robust, conventient methods.
//!
//! This trait assists writing of flatbuffer log messages into a [Dataset].
use super::{DatasetExt, DatasetFlatbuffersExt, NexusHDF5Error, NexusHDF5Result};
use crate::nexus::LogMessage;
use hdf5::{
    Dataset,
    types::{FloatSize, IntSize, TypeDescriptor},
};
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::f144_LogData, ecs_se00_data_generated::se00_SampleEnvironmentData,
};

impl DatasetFlatbuffersExt for Dataset {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn append_f144_value_slice(&self, data: &f144_LogData<'_>) -> NexusHDF5Result<()> {
        let type_descriptor = data.get_type_descriptor()?;
        let error = || NexusHDF5Error::invalid_hdf5_type_conversion(type_descriptor.clone());
        match type_descriptor.clone() {
            TypeDescriptor::Integer(int_size) => match int_size {
                IntSize::U1 => data
                    .value_as_byte()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                IntSize::U2 => data
                    .value_as_short()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                IntSize::U4 => data
                    .value_as_int()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                IntSize::U8 => data
                    .value_as_long()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => data
                    .value_as_ubyte()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                IntSize::U2 => data
                    .value_as_ushort()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                IntSize::U4 => data
                    .value_as_uint()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                IntSize::U8 => data
                    .value_as_ulong()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => data
                    .value_as_float()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
                FloatSize::U8 => data
                    .value_as_double()
                    .ok_or_else(error)
                    .and_then(|x| self.append_value(x.value())),
            },
            TypeDescriptor::VarLenArray(inner_type_descriptor) => {
                match inner_type_descriptor.to_packed_repr() {
                    TypeDescriptor::Integer(int_size) => match int_size {
                        IntSize::U1 => data
                            .value_as_array_byte()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        IntSize::U2 => data
                            .value_as_array_short()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        IntSize::U4 => data
                            .value_as_array_int()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        IntSize::U8 => data
                            .value_as_array_long()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                    },
                    TypeDescriptor::Unsigned(int_size) => match int_size {
                        IntSize::U1 => data
                            .value_as_array_ubyte()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        IntSize::U2 => data
                            .value_as_array_ushort()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        IntSize::U4 => data
                            .value_as_array_uint()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        IntSize::U8 => data
                            .value_as_array_ulong()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                    },
                    TypeDescriptor::Float(float_size) => match float_size {
                        FloatSize::U4 => data
                            .value_as_array_float()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                        FloatSize::U8 => data
                            .value_as_array_double()
                            .and_then(|x| x.value())
                            .ok_or_else(error)
                            .and_then(|x|self.append_slice(x.into_iter().collect::<Vec<_>>().as_slice())),
                    },
                    _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
                }
            }
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
        }
    }

    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn append_se00_value_slice(
        &self,
        data: &se00_SampleEnvironmentData<'_>,
    ) -> NexusHDF5Result<()> {
        let type_descriptor = data.get_type_descriptor()?;
        let error = || NexusHDF5Error::invalid_hdf5_type_conversion(type_descriptor.clone());
        match type_descriptor {
            TypeDescriptor::Integer(int_size) => match int_size {
                IntSize::U1 => self.append_slice(
                    &data
                        .values_as_int_8_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U2 => self.append_slice(
                    &data
                        .values_as_int_16_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U4 => self.append_slice(
                    &data
                        .values_as_int_32_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U8 => self.append_slice(
                    &data
                        .values_as_int_64_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => self.append_slice(
                    &data
                        .values_as_uint_8_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U2 => self.append_slice(
                    &data
                        .values_as_uint_16_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U4 => self.append_slice(
                    &data
                        .values_as_uint_32_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                IntSize::U8 => self.append_slice(
                    &data
                        .values_as_uint_64_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => self.append_slice(
                    &data
                        .values_as_float_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
                FloatSize::U8 => self.append_slice(
                    &data
                        .values_as_double_array()
                        .ok_or_else(error)?
                        .value()
                        .into_iter()
                        .collect::<Vec<_>>(),
                ),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
        }
    }
}
