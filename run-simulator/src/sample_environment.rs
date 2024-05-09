use anyhow::{anyhow, Result};
use std::str::FromStr;
use supermusr_streaming_types::{
    ecs_se00_data_generated::{
        DoubleArray, DoubleArrayArgs, FloatArray, FloatArrayArgs, Int16Array, Int16ArrayArgs,
        Int32Array, Int32ArrayArgs, Int64Array, Int64ArrayArgs, Int8Array, Int8ArrayArgs, Location,
        UInt16Array, UInt16ArrayArgs, UInt32Array, UInt32ArrayArgs, UInt64Array, UInt64ArrayArgs,
        UInt8Array, UInt8ArrayArgs, ValueUnion,
    },
    flatbuffers::{FlatBufferBuilder, Push, UnionWIPOffset, Vector, WIPOffset},
};

pub(crate) fn values_union_type(value_type: &str) -> Result<ValueUnion> {
    Ok(match value_type {
        "int8" => ValueUnion::Int8Array,
        "int16" => ValueUnion::Int16Array,
        "int32" => ValueUnion::Int32Array,
        "int64" => ValueUnion::Int64Array,
        "uint8" => ValueUnion::UInt8Array,
        "uint16" => ValueUnion::UInt16Array,
        "uint32" => ValueUnion::UInt32Array,
        "uint64" => ValueUnion::UInt64Array,
        "float32" => ValueUnion::FloatArray,
        "float64" => ValueUnion::DoubleArray,
        _ => return Err(anyhow!("Invalid HDF5 Type")),
    })
}

pub(crate) fn location(location: &str) -> Result<Location> {
    Ok(match location {
        "unknown" => Location::Unknown,
        "start" => Location::Start,
        "middle" => Location::Middle,
        "end" => Location::End,
        _ => return Err(anyhow!("Invalid Location")),
    })
}

fn to_args<'a, 'fbb: 'a, I: FromStr + Push>(
    fbb: &mut FlatBufferBuilder<'fbb>,
    value: &[String],
) -> Option<WIPOffset<Vector<'a, <I as Push>::Output>>>
where
    <I as Push>::Output: 'fbb,
{
    Some(
        fbb.create_vector(
            value
                .iter()
                .map(|str| str.parse())
                .collect::<Vec<Result<I, <I as FromStr>::Err>>>()
                .into_iter()
                .flatten()
                .collect::<Vec<I>>()
                .as_slice(),
        ),
    )
}

pub(crate) fn make_value(
    fbb: &mut FlatBufferBuilder,
    value_type: ValueUnion,
    value: &[String],
) -> WIPOffset<UnionWIPOffset> {
    match value_type {
        ValueUnion::Int8Array => {
            let args = to_args::<i8>(fbb, value);
            Int8Array::create(fbb, &Int8ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::Int16Array => {
            let args = to_args::<i16>(fbb, value);
            Int16Array::create(fbb, &Int16ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::Int32Array => {
            let args = to_args::<i32>(fbb, value);
            Int32Array::create(fbb, &Int32ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::Int64Array => {
            let args = to_args::<i64>(fbb, value);
            Int64Array::create(fbb, &Int64ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::UInt8Array => {
            let args = to_args::<u8>(fbb, value);
            UInt8Array::create(fbb, &UInt8ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::UInt16Array => {
            let args = to_args::<u16>(fbb, value);
            UInt16Array::create(fbb, &UInt16ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::UInt32Array => {
            let args = to_args::<u32>(fbb, value);
            UInt32Array::create(fbb, &UInt32ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::UInt64Array => {
            let args = to_args::<u64>(fbb, value);
            UInt64Array::create(fbb, &UInt64ArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::FloatArray => {
            let args = to_args::<f32>(fbb, value);
            FloatArray::create(fbb, &FloatArrayArgs { value: args }).as_union_value()
        }
        ValueUnion::DoubleArray => {
            let args = to_args::<f64>(fbb, value);
            DoubleArray::create(fbb, &DoubleArrayArgs { value: args }).as_union_value()
        }
        _ => unreachable!(),
    }
}


#[cfg(test)]
mod tests {
    use supermusr_streaming_types::ecs_se00_data_generated::{finish_se_00_sample_environment_data_buffer, root_as_se_00_sample_environment_data, se00_SampleEnvironmentData, se00_SampleEnvironmentDataArgs};

    use super::*;

    fn process<'a>(fbb : &'a mut FlatBufferBuilder, values_type: ValueUnion, values: WIPOffset<UnionWIPOffset>) -> Result<se00_SampleEnvironmentData<'a>> {
        let selog = se00_SampleEnvironmentDataArgs {
            name: Some(fbb.create_string("")),
            channel: 0,
            packet_timestamp: 0,
            time_delta: 0.0,
            timestamp_location: Location::Unknown,
            values_type,
            values: Some(values),
            timestamps: None,
            message_counter: 0,
        };
        let message = se00_SampleEnvironmentData::create(fbb, &selog);
        finish_se_00_sample_environment_data_buffer(fbb, message);
        let bytes = fbb.finished_data();
        Ok(root_as_se_00_sample_environment_data(bytes)?)
    }

    fn do_array_test<'a>(fbb : &'a mut FlatBufferBuilder, value_type: ValueUnion) -> Result<se00_SampleEnvironmentData<'a>> {
        let test_value = ["2".to_owned(), "3".to_owned()];
        let val = make_value(fbb, value_type, &test_value);
        process(fbb, value_type, val)
    }

    #[test]
    fn make_value_int8_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::Int8Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::Int8Array);
        let array = obj.values_as_int_8_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as i8);
        assert_eq!(array.get(1), 3 as i8);
    }

    #[test]
    fn make_value_int16_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::Int16Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::Int16Array);
        let array = obj.values_as_int_16_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as i16);
        assert_eq!(array.get(1), 3 as i16);
    }

    #[test]
    fn make_value_int32_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::Int32Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::Int32Array);
        let array = obj.values_as_int_32_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as i32);
        assert_eq!(array.get(1), 3 as i32);
    }

    #[test]
    fn make_value_int64_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::Int64Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::Int64Array);
        let array = obj.values_as_int_64_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as i64);
        assert_eq!(array.get(1), 3 as i64);
    }

    #[test]
    fn make_value_uint8_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::UInt8Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::UInt8Array);
        let array = obj.values_as_uint_8_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as u8);
        assert_eq!(array.get(1), 3 as u8);
    }

    #[test]
    fn make_value_uint16_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::UInt16Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::UInt16Array);
        let array = obj.values_as_uint_16_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as u16);
        assert_eq!(array.get(1), 3 as u16);
    }

    #[test]
    fn make_value_uint32_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::UInt32Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::UInt32Array);
        let array = obj.values_as_uint_32_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as u32);
        assert_eq!(array.get(1), 3 as u32);
    }

    #[test]
    fn make_value_uint64_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::UInt64Array).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::UInt64Array);
        let array = obj.values_as_uint_64_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as u64);
        assert_eq!(array.get(1), 3 as u64);
    }

    #[test]
    fn make_value_float_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::FloatArray).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::FloatArray);
        let array = obj.values_as_float_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as f32);
        assert_eq!(array.get(1), 3 as f32);
    }

    #[test]
    fn make_value_double_array() {
        let mut fbb = FlatBufferBuilder::new();
        let obj = do_array_test(&mut fbb, ValueUnion::DoubleArray).unwrap();
        
        assert_eq!(obj.values_type(), ValueUnion::DoubleArray);
        let array = obj.values_as_double_array().unwrap().value();
        assert_eq!(array.len(), 2);
        assert_eq!(array.get(0), 2 as f64);
        assert_eq!(array.get(1), 3 as f64);
    }
}