//! This module implements the traits to extend the hdf5 [Dataset] type to provide robust, conventient methods.
//!
//! This trait assists writing of flatbuffer log messages into a [Dataset].
use super::{DatasetExt, DatasetFlatbuffersExt, NexusHDF5Error, NexusHDF5Result};
use crate::nexus::LogMessage;
use hdf5::{
    Dataset, H5Type,
    types::{FloatSize, IntSize, TypeDescriptor, VarLenArray},
};
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::f144_LogData,
    ecs_se00_data_generated::se00_SampleEnvironmentData,
    flatbuffers::{Follow, Vector},
};

/// Extracts a value of type [Self] from a [f144_LogData] reference, returning the given error if conversion fails.
trait F144HDF5Value: Sized {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error>;
}

impl F144HDF5Value for i8 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_byte().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for i16 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_short().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for i32 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_int().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for i64 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_long().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for u8 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_ubyte().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for u16 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_ushort().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for u32 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_uint().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for u64 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_ulong().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for f32 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_float().ok_or_else(error)?.value())
    }
}

impl F144HDF5Value for f64 {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        Ok(data.value_as_double().ok_or_else(error)?.value())
    }
}

/// Implements associated function which automates repetative conversion operations in implementations of [F144HDF5Value] for [VarLenArray].
trait F144HDF5VarLenArrayHelper<'a>: Sized {
    type Output: H5Type + Copy + Follow<'a>;

    /// Converts a flatbuffer [Vector] into a [VarLenArray<Self>].
    fn var_len_array_or_then(
        self,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<VarLenArray<Self::Output>, NexusHDF5Error>;
}

impl<'a, T> F144HDF5VarLenArrayHelper<'a> for Option<Vector<'a, T>>
where
    T: H5Type + Copy + Follow<'a, Inner = T>,
{
    type Output = T;
    fn var_len_array_or_then(
        self,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<VarLenArray<T>, NexusHDF5Error> {
        self.map(|vec| VarLenArray::from_slice(vec.into_iter().collect::<Vec<_>>().as_slice()))
            .ok_or_else(error)
    }
}

impl F144HDF5Value for VarLenArray<i8> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_byte()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<i16> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_short()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<i32> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_int()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<i64> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_long()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<u8> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_ubyte()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<u16> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_ushort()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<u32> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_uint()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<u64> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_ulong()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<f32> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_float()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

impl F144HDF5Value for VarLenArray<f64> {
    fn f144_value_or_then(
        data: &f144_LogData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Self, NexusHDF5Error> {
        data.value_as_array_double()
            .and_then(|val| val.value())
            .var_len_array_or_then(error)
    }
}

/// Extracts a value of type [Self] from a [se00_SampleEnvironmentData] reference, returning the given error if conversion fails.
trait Se00HDF5Value: Sized {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error>;
}

/// Implements associated function which automates repetative conversion operations in implementations of [Se00HDF5Value] for [VarLenArray].
trait Se00HDF5VarLenArrayHelper<'a>: Sized {
    type Output: H5Type + Copy + Follow<'a>;

    /// Converts a flatbuffer [Vector] into a [VarLenArray<Self>].
    fn vec_or_then(
        self,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self::Output>, NexusHDF5Error>;
}

impl<'a, T> Se00HDF5VarLenArrayHelper<'a> for Option<Vector<'a, T>>
where
    T: H5Type + Copy + Follow<'a, Inner = T>,
{
    type Output = T;
    fn vec_or_then(self, error: impl Fn() -> NexusHDF5Error) -> Result<Vec<T>, NexusHDF5Error> {
        Ok(self.ok_or_else(error)?.into_iter().collect::<Vec<_>>())
    }
}

impl Se00HDF5Value for i8 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_int_8_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for i16 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_int_16_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for i32 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_int_32_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for i64 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_int_64_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for u8 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_uint_8_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for u16 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_uint_16_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for u32 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_uint_32_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for u64 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_uint_64_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for f32 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_float_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl Se00HDF5Value for f64 {
    fn se00_value_or_then(
        data: &se00_SampleEnvironmentData<'_>,
        error: impl Fn() -> NexusHDF5Error,
    ) -> Result<Vec<Self>, NexusHDF5Error> {
        data.values_as_double_array()
            .map(|val| val.value())
            .vec_or_then(error)
    }
}

impl DatasetFlatbuffersExt for Dataset {
    #[tracing::instrument(skip_all, level = "debug", err(level = "warn"))]
    fn append_f144_value_slice(&self, data: &f144_LogData<'_>) -> NexusHDF5Result<()> {
        let type_descriptor = data.get_type_descriptor()?;
        let error = || NexusHDF5Error::invalid_hdf5_type_conversion(type_descriptor.clone());
        match type_descriptor.clone() {
            TypeDescriptor::Integer(int_size) => match int_size {
                IntSize::U1 => self.append_value(i8::f144_value_or_then(data, error)?),
                IntSize::U2 => self.append_value(i16::f144_value_or_then(data, error)?),
                IntSize::U4 => self.append_value(i32::f144_value_or_then(data, error)?),
                IntSize::U8 => self.append_value(i64::f144_value_or_then(data, error)?),
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => self.append_value(u8::f144_value_or_then(data, error)?),
                IntSize::U2 => self.append_value(u16::f144_value_or_then(data, error)?),
                IntSize::U4 => self.append_value(u32::f144_value_or_then(data, error)?),
                IntSize::U8 => self.append_value(u64::f144_value_or_then(data, error)?),
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => self.append_value(f32::f144_value_or_then(data, error)?),
                FloatSize::U8 => self.append_value(f64::f144_value_or_then(data, error)?),
            },
            TypeDescriptor::VarLenArray(inner_type_descriptor) => {
                match inner_type_descriptor.to_packed_repr() {
                    TypeDescriptor::Integer(int_size) => {
                        match int_size {
                            IntSize::U1 => self
                                .append_value(VarLenArray::<i8>::f144_value_or_then(data, error)?),
                            IntSize::U2 => self
                                .append_value(VarLenArray::<i16>::f144_value_or_then(data, error)?),
                            IntSize::U4 => self
                                .append_value(VarLenArray::<i32>::f144_value_or_then(data, error)?),
                            IntSize::U8 => self
                                .append_value(VarLenArray::<i64>::f144_value_or_then(data, error)?),
                        }
                    }
                    TypeDescriptor::Unsigned(int_size) => {
                        match int_size {
                            IntSize::U1 => self
                                .append_value(VarLenArray::<u8>::f144_value_or_then(data, error)?),
                            IntSize::U2 => self
                                .append_value(VarLenArray::<u16>::f144_value_or_then(data, error)?),
                            IntSize::U4 => self
                                .append_value(VarLenArray::<u32>::f144_value_or_then(data, error)?),
                            IntSize::U8 => self
                                .append_value(VarLenArray::<u64>::f144_value_or_then(data, error)?),
                        }
                    }
                    TypeDescriptor::Float(float_size) => {
                        match float_size {
                            FloatSize::U4 => self
                                .append_value(VarLenArray::<f32>::f144_value_or_then(data, error)?),
                            FloatSize::U8 => self
                                .append_value(VarLenArray::<f64>::f144_value_or_then(data, error)?),
                        }
                    }
                    _ => unreachable!(
                        "Unreachable HDF5 TypeDescriptor reached, this should never happen"
                    ),
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
                IntSize::U1 => self.append_slice(&i8::se00_value_or_then(data, error)?),
                IntSize::U2 => self.append_slice(&i16::se00_value_or_then(data, error)?),
                IntSize::U4 => self.append_slice(&i32::se00_value_or_then(data, error)?),
                IntSize::U8 => self.append_slice(&i64::se00_value_or_then(data, error)?),
            },
            TypeDescriptor::Unsigned(int_size) => match int_size {
                IntSize::U1 => self.append_slice(&u8::se00_value_or_then(data, error)?),
                IntSize::U2 => self.append_slice(&u16::se00_value_or_then(data, error)?),
                IntSize::U4 => self.append_slice(&u32::se00_value_or_then(data, error)?),
                IntSize::U8 => self.append_slice(&u64::se00_value_or_then(data, error)?),
            },
            TypeDescriptor::Float(float_size) => match float_size {
                FloatSize::U4 => self.append_slice(&f32::se00_value_or_then(data, error)?),
                FloatSize::U8 => self.append_slice(&f64::se00_value_or_then(data, error)?),
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached, this should never happen"),
        }
    }
}
