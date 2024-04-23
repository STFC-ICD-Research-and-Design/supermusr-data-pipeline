use super::{
    add_new_group_to, create_resizable_2d_dataset, create_resizable_2d_dataset_dyn_type,
    create_resizable_dataset,
};
use crate::nexus::{nexus_class as NX, NexusSettings};
use anyhow::{anyhow, Result};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset, Group,
};
use ndarray::s;
use supermusr_streaming_types::ecs_se00_data_generated::{se00_SampleEnvironmentData, ValueUnion};
use tracing::debug;

#[derive(Debug)]
pub(crate) struct SeLog {
    num_selogs: usize,

    channel: Dataset,
    packet_timestamp: Dataset,
    time_delta: Dataset,
    timestamp_location: Dataset,
    values: Dataset,
    timestamps: Dataset,
    message_counter: Dataset,
}
impl SeLog {
    #[tracing::instrument(fields(class = "SELog"))]
    pub(crate) fn new(parent: &Group, settings: &NexusSettings) -> Result<Self> {
        let selog = add_new_group_to(parent, "selog", NX::SELOG)?;
        let channel = create_resizable_dataset::<i32>(&selog, "channel", 0, 32)?;
        let packet_timestamp = create_resizable_dataset::<i64>(&selog, "packet_timestamp", 0, 32)?;
        let time_delta = create_resizable_dataset::<f64>(&selog, "time_delta", 0, 32)?;
        let timestamp_location =
            create_resizable_dataset::<u8>(&selog, "timestamp_location", 0, 32)?;

        let values = create_resizable_2d_dataset_dyn_type(
            &selog,
            "values",
            &settings.sample_env.data_type,
            (0, 0),
            (32, 32),
        )?;
        let timestamps =
            create_resizable_2d_dataset::<i64>(&selog, "timestamps", (0, 0), (32, 32))?;

        let message_counter = create_resizable_dataset::<i64>(&selog, "message_counter", 0, 32)?;

        Ok(Self {
            num_selogs: 0,
            channel,
            packet_timestamp,
            time_delta,
            timestamp_location,
            values,
            timestamps,
            message_counter,
        })
    }

    #[tracing::instrument(fields(class = "SELog"))]
    pub(crate) fn open(parent: &Group) -> Result<Self> {
        let selog = parent.group("selog")?;
        let channel = selog.dataset("channel")?;
        let packet_timestamp = selog.dataset("packet_timestamp")?;
        let time_delta = selog.dataset("time_delta")?;
        let timestamp_location = selog.dataset("timestamp_location")?;
        let values = selog.dataset("values")?;
        let timestamps = selog.dataset("timestamps")?;
        let message_counter = selog.dataset("message_counter")?;

        Ok(Self {
            num_selogs: 0,
            channel,
            packet_timestamp,
            time_delta,
            timestamp_location,
            values,
            timestamps,
            message_counter,
        })
    }

    fn get_hdf5_type(fb_type: ValueUnion) -> Result<TypeDescriptor> {
        let datatype = match fb_type {
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
                return Err(anyhow!(
                    "Invalid flatbuffers selogdata type {}",
                    t.variant_name().unwrap()
                ))
            }
        };
        Ok(datatype)
    }

    #[tracing::instrument(fields(class = "SELog", message_number, num_events))]
    pub(crate) fn push_selogdata(
        &mut self,
        selogdata: se00_SampleEnvironmentData,
        settings: &NexusSettings,
    ) -> Result<()> {
        tracing::Span::current().record("selog_number", self.num_selogs);
        debug!("{:?}", selogdata.values_type());
        let datatype = Self::get_hdf5_type(selogdata.values_type())?;
        if datatype != settings.sample_env.data_type {
            return Err(anyhow!("Incorrect Datatype"));
        }

        let slice = s![self.num_selogs..(self.num_selogs + 1)];

        self.channel.resize(self.num_selogs + 1).unwrap();
        self.channel.write_slice(&[selogdata.channel()], slice)?;

        self.packet_timestamp.resize(self.num_selogs + 1).unwrap();
        self.packet_timestamp
            .write_slice(&[selogdata.packet_timestamp()], slice)?;

        self.time_delta.resize(self.num_selogs + 1).unwrap();
        self.time_delta
            .write_slice(&[selogdata.time_delta()], slice)?;

        self.timestamp_location.resize(self.num_selogs + 1).unwrap();
        self.timestamp_location
            .write_slice(&[selogdata.timestamp_location().0], slice)?;

        let array_slice = s![
            self.num_selogs..(self.num_selogs + 1),
            0..settings.sample_env.array_length
        ];

        self.timestamps.resize(self.num_selogs + 1).unwrap();
        self.timestamps.write_slice(
            &selogdata.timestamps().unwrap().iter().collect::<Vec<_>>(),
            array_slice,
        )?;

        self.values
            .resize((self.num_selogs + 1, settings.sample_env.array_length))
            .unwrap();
        match settings.sample_env.data_type {
            TypeDescriptor::Integer(sz) => match sz {
                IntSize::U1 => self.values.write_slice(
                    &selogdata
                        .values_as_int_8_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                IntSize::U2 => self.values.write_slice(
                    &selogdata
                        .values_as_int_16_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                IntSize::U4 => self.values.write_slice(
                    &selogdata
                        .values_as_int_32_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                IntSize::U8 => self.values.write_slice(
                    &selogdata
                        .values_as_int_64_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
            },
            TypeDescriptor::Unsigned(sz) => match sz {
                IntSize::U1 => self.values.write_slice(
                    &selogdata
                        .values_as_uint_8_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                IntSize::U2 => self.values.write_slice(
                    &selogdata
                        .values_as_uint_16_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                IntSize::U4 => self.values.write_slice(
                    &selogdata
                        .values_as_uint_32_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                IntSize::U8 => self.values.write_slice(
                    &selogdata
                        .values_as_uint_64_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
            },
            TypeDescriptor::Float(sz) => match sz {
                FloatSize::U4 => self.values.write_slice(
                    &selogdata
                        .values_as_float_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
                FloatSize::U8 => self.values.write_slice(
                    &selogdata
                        .values_as_double_array()
                        .unwrap()
                        .value()
                        .iter()
                        .collect::<Vec<_>>(),
                    array_slice,
                ),
            },
            _ => {
                return Err(anyhow!(
                    "Invalid HDF5 type: {}",
                    settings.log.data_type.to_string()
                ))
            }
        }?;

        self.message_counter.resize(self.num_selogs + 1).unwrap();
        self.message_counter
            .write_slice(&[selogdata.message_counter()], slice)?;

        self.num_selogs += 1;

        Ok(())
    }
}
