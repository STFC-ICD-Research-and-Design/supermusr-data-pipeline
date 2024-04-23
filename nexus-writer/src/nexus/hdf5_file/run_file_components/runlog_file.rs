use super::{add_new_group_to, create_resizable_2d_dataset_dyn_type, create_resizable_dataset};
use crate::nexus::{nexus_class as NX, NexusSettings, VarArrayTypeSettings};
use anyhow::{anyhow, Result};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, Group, H5Type,
};
use ndarray::{s, Array2, Dim, SliceInfo, SliceInfoElem};
use std::fmt::Debug;
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::{f144_LogData, Value},
    flatbuffers::{Follow, Vector},
};
use tracing::{debug, error, trace};

const TRACING_CLASS: &str = "NexusWriter::RunLog";

type Slice2D = SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 2]>>;

#[derive(Debug)]
pub(crate) struct RunLog {
    num_runlogs: usize,

    timestamp: Dataset,
    value: Dataset,
}

impl RunLog {
    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn new(parent: &Group, settings: &NexusSettings) -> Result<Self> {
        let logs = add_new_group_to(parent, "runlog", NX::RUNLOG)?;
        let timestamp = create_resizable_dataset::<i64>(&logs, "timestamp", 0, 32)?;
        let value = create_resizable_2d_dataset_dyn_type(
            &logs,
            "value",
            &settings.log.data_type,
            (0, 0),
            (32, 32),
        )?;

        Ok(Self {
            num_runlogs: 0,
            timestamp,
            value,
        })
    }

    #[tracing::instrument(fields(class = TRACING_CLASS))]
    pub(crate) fn open(parent: &Group) -> Result<Self> {
        let logs = parent.group("runlog")?;
        let timestamp = logs.dataset("timestamp")?;
        let value = logs.dataset("value")?;

        Ok(Self {
            num_runlogs: timestamp.size(),
            timestamp,
            value,
        })
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
            Value::ArrayByte => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Integer(IntSize::U1)))
            }
            Value::ArrayUByte => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Unsigned(IntSize::U1)))
            }
            Value::ArrayShort => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Integer(IntSize::U2)))
            }
            Value::ArrayUShort => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Unsigned(IntSize::U2)))
            }
            Value::ArrayInt => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Integer(IntSize::U4)))
            }
            Value::ArrayUInt => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Unsigned(IntSize::U4)))
            }
            Value::ArrayLong => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Integer(IntSize::U8)))
            }
            Value::ArrayULong => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Unsigned(IntSize::U8)))
            }
            Value::ArrayFloat => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Float(FloatSize::U4)))
            }
            Value::ArrayDouble => {
                TypeDescriptor::VarLenArray(Box::new(TypeDescriptor::Float(FloatSize::U8)))
            }
            t => {
                return Err(anyhow!(
                    "Invalid flatbuffers logdata type {}",
                    t.variant_name().unwrap()
                ))
            }
        };
        Ok(datatype)
    }

    fn write_generic_slice_array<'a, T: Debug + Follow<'a> + H5Type>(
        &mut self,
        value: Option<Vector<'a, T>>,
        slice: &Slice2D,
        array_size: usize,
    ) -> Result<()>
    where
        <T as Follow<'a>>::Inner: Debug + H5Type,
    {
        let value = Array2::from_shape_vec((1, array_size), value.unwrap().iter().collect())?;
        trace!("Value(s): {value:?}");
        self.value.write_slice(&value, *slice)?;
        Ok(())
    }

    fn write_value_slice_array(
        &mut self,
        logdata: &f144_LogData,
        slice: &Slice2D,
        type_descriptor: &TypeDescriptor,
        array_size: usize,
    ) -> Result<()> {
        trace!("Type: {type_descriptor}");
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => self.write_generic_slice_array(
                    logdata.value_as_array_byte().unwrap().value(),
                    slice,
                    array_size,
                ),
                IntSize::U2 => self.write_generic_slice_array(
                    logdata.value_as_array_short().unwrap().value(),
                    slice,
                    array_size,
                ),
                IntSize::U4 => self.write_generic_slice_array(
                    logdata.value_as_array_int().unwrap().value(),
                    slice,
                    array_size,
                ),
                IntSize::U8 => self.write_generic_slice_array(
                    logdata.value_as_array_long().unwrap().value(),
                    slice,
                    array_size,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => self.write_generic_slice_array(
                    logdata.value_as_array_ubyte().unwrap().value(),
                    slice,
                    array_size,
                ),
                IntSize::U2 => self.write_generic_slice_array(
                    logdata.value_as_array_ushort().unwrap().value(),
                    slice,
                    array_size,
                ),
                IntSize::U4 => self.write_generic_slice_array(
                    logdata.value_as_array_uint().unwrap().value(),
                    slice,
                    array_size,
                ),
                IntSize::U8 => self.write_generic_slice_array(
                    logdata.value_as_array_ulong().unwrap().value(),
                    slice,
                    array_size,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => self.write_generic_slice_array(
                    logdata.value_as_array_float().unwrap().value(),
                    slice,
                    array_size,
                ),
                FloatSize::U8 => self.write_generic_slice_array(
                    logdata.value_as_array_double().unwrap().value(),
                    slice,
                    array_size,
                ),
            },
            _ => {
                return Err(anyhow!(
                    "Invalid HDF5 array type: {}",
                    type_descriptor.to_string()
                ))
            }
        }?;
        Ok(())
    }

    fn write_generic_slice_scalar<T: Debug + H5Type>(
        &mut self,
        value: T,
        slice: &Slice2D,
    ) -> Result<()> {
        let value = Array2::from_shape_vec((1, 1), vec![value])?;
        trace!("Value(s): {value:?}");
        self.value.write_slice(&value, *slice)?;
        Ok(())
    }

    fn write_value_slice_scalar(
        &mut self,
        logdata: &f144_LogData,
        slice: &Slice2D,
        type_descriptor: &TypeDescriptor,
    ) -> Result<()> {
        trace!("Scalar Type: {type_descriptor}");
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => {
                    self.write_generic_slice_scalar(logdata.value_as_byte().unwrap().value(), slice)
                }
                IntSize::U2 => self
                    .write_generic_slice_scalar(logdata.value_as_short().unwrap().value(), slice),
                IntSize::U4 => {
                    self.write_generic_slice_scalar(logdata.value_as_int().unwrap().value(), slice)
                }
                IntSize::U8 => {
                    self.write_generic_slice_scalar(logdata.value_as_long().unwrap().value(), slice)
                }
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => self
                    .write_generic_slice_scalar(logdata.value_as_ubyte().unwrap().value(), slice),
                IntSize::U2 => self
                    .write_generic_slice_scalar(logdata.value_as_ushort().unwrap().value(), slice),
                IntSize::U4 => {
                    self.write_generic_slice_scalar(logdata.value_as_uint().unwrap().value(), slice)
                }
                IntSize::U8 => self
                    .write_generic_slice_scalar(logdata.value_as_ulong().unwrap().value(), slice),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => self
                    .write_generic_slice_scalar(logdata.value_as_float().unwrap().value(), slice),
                FloatSize::U8 => self
                    .write_generic_slice_scalar(logdata.value_as_double().unwrap().value(), slice),
            },
            _ => {
                return Err(anyhow!(
                    "Invalid HDF5 array type: {}",
                    type_descriptor.to_string()
                ))
            }
        }?;
        Ok(())
    }

    fn write_value_slice(
        &mut self,
        logdata: &f144_LogData,
        slice: &Slice2D,
        settings: &VarArrayTypeSettings,
    ) -> Result<()> {
        match &settings.data_type {
            TypeDescriptor::VarLenArray(ref t) => {
                trace!("Variable Length Array");
                self.write_value_slice_array(logdata, slice, t.as_ref(), settings.array_length)?
            }
            TypeDescriptor::FixedArray(ref t, sz) => {
                trace!("Fixed Size Array of Length {}", sz);
                if *sz != settings.array_length {
                    error!("Expected Array Length of {}", settings.array_length);
                }
                self.write_value_slice_array(logdata, slice, t.as_ref(), settings.array_length)?
            }
            t => {
                trace!("Scalar");
                self.write_value_slice_scalar(logdata, slice, t)?
            }
        };
        Ok(())
    }

    #[tracing::instrument(fields(class = TRACING_CLASS, runlog_number))]
    pub(crate) fn push_logdata(
        &mut self,
        logdata: &f144_LogData,
        settings: &VarArrayTypeSettings,
    ) -> Result<()> {
        tracing::Span::current().record("runlog_number", self.num_runlogs);
        debug!("{:?}", logdata.value_type());
        let datatype = Self::get_hdf5_type(logdata.value_type())?;
        if datatype != settings.data_type {
            return Err(anyhow!("Incorrect Datatype"));
        }

        self.timestamp.resize(self.num_runlogs + 1).unwrap();
        let slice = s![self.num_runlogs..(self.num_runlogs + 1)];
        debug!(
            "Timestamp Slice: {slice:?}, Value: {0:?}",
            logdata.timestamp()
        );
        self.timestamp.write_slice(&[logdata.timestamp()], slice)?;

        self.value
            .resize((self.num_runlogs + 1, settings.array_length))
            .unwrap();

        let slice = s![
            self.num_runlogs..(self.num_runlogs + 1),
            0..settings.array_length
        ];
        debug!("Values Slice: {slice:?}");
        self.write_value_slice(logdata, &slice, settings)?;

        self.num_runlogs += 1;

        Ok(())
    }
}
