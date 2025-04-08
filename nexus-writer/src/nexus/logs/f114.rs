use crate::{
    error::FlatBufferInvalidDataTypeContext,
    hdf5_handlers::{
        ConvertResult, DatasetExt, DatasetFlatbuffersExt, NexusHDF5Error, NexusHDF5Result,
    },
    run_engine::NexusDateTime,
};
use hdf5::{
    types::{FloatSize, IntSize, TypeDescriptor},
    Dataset,
};
use supermusr_streaming_types::ecs_f144_logdata_generated::{f144_LogData, Value};

use super::{adjust_nanoseconds_by_origin_to_sec, remove_prefixes, LogMessage};

impl<'a> LogMessage<'a> for f144_LogData<'a> {
    fn get_name(&self) -> String {
        remove_prefixes(self.source_name())
    }

    fn get_type_descriptor(&self) -> NexusHDF5Result<TypeDescriptor> {
        let error = |value: Value| {
            NexusHDF5Error::new_flatbuffer_invalid_data_type(
                FlatBufferInvalidDataTypeContext::RunLog,
                value
                    .variant_name()
                    .map(ToOwned::to_owned)
                    .unwrap_or_default(),
            )
        };
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
            value => return Err(error(value)),
        };
        Ok(datatype)
    }

    fn append_timestamps_to(
        &self,
        dataset: &Dataset,
        origin_time: &NexusDateTime,
    ) -> NexusHDF5Result<()> {
        dataset
            .append_value(adjust_nanoseconds_by_origin_to_sec(
                self.timestamp(),
                origin_time,
            ))
            .err_dataset(dataset)
    }

    fn append_values_to(&self, dataset: &Dataset) -> NexusHDF5Result<()> {
        if dataset.as_datatype()?.to_descriptor()? != self.get_type_descriptor()? {
            return Err(NexusHDF5Error::new_invalid_hdf5_type_conversion(
                self.get_type_descriptor()?,
            ))
            .err_dataset(dataset);
        }
        dataset.append_f144_value_slice(self).err_dataset(dataset)
    }
}
