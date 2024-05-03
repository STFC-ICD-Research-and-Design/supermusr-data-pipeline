use super::{
    add_new_group_to,
    timeseries_file::{Slice1D, TimeSeriesEntry, TimeSeriesOwner},
};
use crate::nexus::{
    hdf5_file::run_file_components::timeseries_file::TimeSeries, nexus_class as NX,
};
use anyhow::{anyhow, bail, Result};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, Group,
};
use std::fmt::Debug;
use supermusr_streaming_types::ecs_f144_logdata_generated::{f144_LogData, Value};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct RunLog {
    parent: Group,
}

impl RunLog {
    #[tracing::instrument]
    pub(crate) fn new_runlog(parent: &Group) -> Result<Self> {
        let logs = add_new_group_to(parent, "runlog", NX::RUNLOG)?;
        Ok(Self { parent: logs })
    }

    #[tracing::instrument]
    pub(crate) fn open_runlog(parent: &Group) -> Result<Self> {
        let parent = parent.group("runlog")?;
        Ok(Self { parent })
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn push_logdata_to_runlog(&mut self, logdata: &f144_LogData) -> Result<()> {
        debug!("Type: {0:?}", logdata.value_type());

        let mut run_log_timeseries = match self.parent.group(logdata.source_name()) {
            Ok(log) => TimeSeries::<RunLog>::open_from_timeseries_group(&log),
            Err(err) => TimeSeries::<RunLog>::new_timeseries(
                &self.parent,
                logdata.source_name().to_owned(),
                get_hdf5_type(logdata.value_type()).map_err(|e| e.context(err))?,
            ),
        }?;
        run_log_timeseries.push_runlog_timeseries(logdata)?;
        Ok(())
    }
}

impl<'a> TimeSeriesEntry<'a> for f144_LogData<'a> {
    fn timestamp(&self) -> i64 {
        self.timestamp()
    }
}

impl<'a> TimeSeriesOwner<'a> for RunLog {
    type DataSource = f144_LogData<'a>;

    #[tracing::instrument]
    fn write_data_value_to_dataset_slice(
        type_descriptor: &TypeDescriptor,
        source: &Self::DataSource,
        target: &mut Dataset,
        slice: Slice1D,
    ) -> Result<()> {
        let error = anyhow!("Could not convert value to type {type_descriptor:?}");
        match type_descriptor {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => {
                    target.write_slice(&[source.value_as_byte().ok_or(error)?.value()], slice)
                }
                IntSize::U2 => {
                    target.write_slice(&[source.value_as_short().ok_or(error)?.value()], slice)
                }
                IntSize::U4 => {
                    target.write_slice(&[source.value_as_int().ok_or(error)?.value()], slice)
                }
                IntSize::U8 => {
                    target.write_slice(&[source.value_as_long().ok_or(error)?.value()], slice)
                }
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => {
                    target.write_slice(&[source.value_as_ubyte().ok_or(error)?.value()], slice)
                }
                IntSize::U2 => {
                    target.write_slice(&[source.value_as_ushort().ok_or(error)?.value()], slice)
                }
                IntSize::U4 => {
                    target.write_slice(&[source.value_as_uint().ok_or(error)?.value()], slice)
                }
                IntSize::U8 => {
                    target.write_slice(&[source.value_as_ulong().ok_or(error)?.value()], slice)
                }
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => {
                    target.write_slice(&[source.value_as_float().ok_or(error)?.value()], slice)
                }
                FloatSize::U8 => {
                    target.write_slice(&[source.value_as_double().ok_or(error)?.value()], slice)
                }
            },
            _ => unreachable!("Unreachable HDF5 TypeDescriptor reached"),
        }?;
        Ok(())
    }
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
