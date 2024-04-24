use anyhow::{anyhow, Result};
use std::str::FromStr;
use supermusr_streaming_types::{
    ecs_se00_data_generated::{
        DoubleArray, DoubleArrayArgs, FloatArray, FloatArrayArgs, Int16Array, Int16ArrayArgs, Int32Array, Int32ArrayArgs, Int64Array, Int64ArrayArgs, Int8Array, Int8ArrayArgs, Location, UInt16Array, UInt16ArrayArgs, UInt32Array, UInt32ArrayArgs, UInt64Array, UInt64ArrayArgs, UInt8Array, UInt8ArrayArgs, ValueUnion
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
