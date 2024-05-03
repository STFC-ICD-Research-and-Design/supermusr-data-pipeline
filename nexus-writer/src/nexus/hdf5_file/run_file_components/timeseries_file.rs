use super::{add_new_group_to, create_resizable_dataset};
use crate::nexus::nexus_class as NX;
use anyhow::{anyhow, bail, Result};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, DatasetBuilderEmpty, Group, SimpleExtents,
};
use ndarray::{s, Dim, SliceInfo, SliceInfoElem};
use std::{fmt::Debug, marker::PhantomData};
use tracing::debug;

pub(super) type Slice1D = SliceInfo<[SliceInfoElem; 1], Dim<[usize; 1]>, Dim<[usize; 1]>>;

pub(super) trait TimeSeriesEntry<'a>: Debug {
    fn timestamp(&self) -> i64;
}

pub(super) trait TimeSeriesOwner<'a> {
    type DataSource: TimeSeriesEntry<'a>;

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
    pub(super) fn push_runlog_timeseries(&mut self, data: &OWNER::DataSource) -> Result<()> {
        let size = self.timestamps.size();

        self.timestamps.resize(size + 1).unwrap();
        let slice = s![size..(size + 1)];
        debug!("Timestamp Slice: {slice:?}, Value: {0:?}", data.timestamp());
        self.timestamps.write_slice(&[data.timestamp()], slice)?;

        self.values.resize(size + 1).unwrap();
        debug!("Values Slice: {slice:?}");
        OWNER::write_data_value_to_dataset_slice(
            &self.type_descriptor,
            data,
            &mut self.values,
            slice,
        )?;
        Ok(())
    }
}

fn get_dataset_builder(
    parent: &Group,
    type_descriptor: &TypeDescriptor,
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
