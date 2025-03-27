use hdf5::{types::{FloatSize, IntSize, TypeDescriptor}, Dataset};
use supermusr_streaming_types::{ecs_f144_logdata_generated::f144_LogData, ecs_se00_data_generated::se00_SampleEnvironmentData};

use crate::nexus::LogMessage;

use super::{DatasetExt, NexusHDF5Error, NexusHDF5Result};

trait DatasetFlatbuffersExt {
    fn append_f144_value_slice<'a>(&self, data: &f144_LogData<'a>) -> NexusHDF5Result<()>;
    fn append_se00_value_slice<'a>(&self, data: &se00_SampleEnvironmentData<'a>) -> NexusHDF5Result<()>;
}

impl DatasetFlatbuffersExt for Dataset {
    fn append_f144_value_slice<'a>(&self, data: &f144_LogData<'a>) -> NexusHDF5Result<()> {
        let type_descriptor = data.get_type_descriptor()?;
        let error = || NexusHDF5Error::new_invalid_hdf5_type_conversion(type_descriptor.clone());
        match type_descriptor {
            TypeDescriptor::Integer(int_size) => match int_size {
                IntSize::U1 => {
                    self.append_slice(&[data.value_as_byte().ok_or_else(error)?.value()])
                }
                IntSize::U2 => {
                    self.append_slice(&[data.value_as_short().ok_or_else(error)?.value()])
                }
                IntSize::U4 => {
                    self.append_slice(&[data.value_as_int().ok_or_else(error)?.value()])
                }
                IntSize::U8 => {
                    self.append_slice(&[data.value_as_long().ok_or_else(error)?.value()])
                }
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => {
                    self.append_slice(&[data.value_as_ubyte().ok_or_else(error)?.value()])
                }
                IntSize::U2 => {
                    self.append_slice(&[data.value_as_ushort().ok_or_else(error)?.value()])
                }
                IntSize::U4 => {
                    self.append_slice(&[data.value_as_uint().ok_or_else(error)?.value()])
                }
                IntSize::U8 => {
                    self.append_slice(&[data.value_as_ulong().ok_or_else(error)?.value()])
                }
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => {
                    self.append_slice(&[data.value_as_float().ok_or_else(error)?.value()])
                }
                FloatSize::U8 => {
                    self.append_slice(&[data.value_as_double().ok_or_else(error)?.value()])
                }
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
        }
    }

    fn append_se00_value_slice<'a>(&self, data: &se00_SampleEnvironmentData<'a>) -> NexusHDF5Result<()> {
        let type_descriptor = data.get_type_descriptor()?;
        let error = || NexusHDF5Error::new_invalid_hdf5_type_conversion(type_descriptor.clone());
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