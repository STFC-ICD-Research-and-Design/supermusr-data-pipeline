use anyhow::{anyhow, Result};
use std::str::FromStr;
use supermusr_streaming_types::{
    ecs_f144_logdata_generated::{
        ArrayByte, ArrayByteArgs, ArrayDouble, ArrayDoubleArgs, ArrayFloat, ArrayFloatArgs,
        ArrayInt, ArrayIntArgs, ArrayLong, ArrayLongArgs, ArrayShort, ArrayShortArgs, ArrayUByte,
        ArrayUByteArgs, ArrayUInt, ArrayUIntArgs, ArrayULong, ArrayULongArgs, ArrayUShort,
        ArrayUShortArgs, Value,
    },
    flatbuffers::{FlatBufferBuilder, Push, UnionWIPOffset, Vector, WIPOffset},
};

pub(crate) fn value_type(value_type: &str) -> Result<Value> {
    Ok(match value_type {
        "int8" => Value::Byte,
        "int16" => Value::Short,
        "int32" => Value::Int,
        "int64" => Value::Long,
        "uint8" => Value::UByte,
        "uint16" => Value::UShort,
        "uint32" => Value::UInt,
        "uint64" => Value::ULong,
        "float32" => Value::Float,
        "float64" => Value::Double,
        "[int8]" => Value::ArrayByte,
        "[int16]" => Value::ArrayShort,
        "[int32]" => Value::ArrayInt,
        "[int64]" => Value::ArrayLong,
        "[uint8]" => Value::ArrayUByte,
        "[uint16]" => Value::ArrayUShort,
        "[uint32]" => Value::ArrayUInt,
        "[uint64]" => Value::ArrayULong,
        "[float32]" => Value::ArrayFloat,
        "[float64]" => Value::ArrayDouble,
        _ => return Err(anyhow!("Invalid HDF5 Type")),
    })
}

type GenericFBVector<'a,I> = WIPOffset<Vector<'a, <I as Push>::Output>>;

fn to_args<'a, 'fbb: 'a, I: FromStr + Push>(
    fbb: &mut FlatBufferBuilder<'fbb>,
    value: &[String],
) -> Result<Option<GenericFBVector<'a, I>>, <I as FromStr>::Err>
where
    <I as Push>::Output: 'fbb,
{
    Ok(Some(
        fbb.create_vector(
            value
                .iter()
                .map(|str| str.parse())
                .collect::<Result<Vec<I>, <I as FromStr>::Err>>()?
                .as_slice(),
        ),
    ))
}

pub(crate) fn make_value(
    fbb: &mut FlatBufferBuilder,
    value_type: Value,
    value: &[String],
) -> Result<WIPOffset<UnionWIPOffset>> {
    Ok(match value_type {
        Value::Byte => {
            let args = to_args::<i8>(fbb, value)?;
            ArrayByte::create(fbb, &ArrayByteArgs { value: args }).as_union_value()
        }
        Value::Short => {
            let args = to_args::<i16>(fbb, value)?;
            ArrayShort::create(fbb, &ArrayShortArgs { value: args }).as_union_value()
        }
        Value::Int => {
            let args = to_args::<i32>(fbb, value)?;
            ArrayInt::create(fbb, &ArrayIntArgs { value: args }).as_union_value()
        }
        Value::Long => {
            let args = to_args::<i64>(fbb, value)?;
            ArrayLong::create(fbb, &ArrayLongArgs { value: args }).as_union_value()
        }
        Value::UByte => {
            let args = to_args::<u8>(fbb, value)?;
            ArrayUByte::create(fbb, &ArrayUByteArgs { value: args }).as_union_value()
        }
        Value::UShort => {
            let args = to_args::<u16>(fbb, value)?;
            ArrayUShort::create(fbb, &ArrayUShortArgs { value: args }).as_union_value()
        }
        Value::UInt => {
            let args = to_args::<u32>(fbb, value)?;
            ArrayUInt::create(fbb, &ArrayUIntArgs { value: args }).as_union_value()
        }
        Value::ULong => {
            let args = to_args::<u64>(fbb, value)?;
            ArrayULong::create(fbb, &ArrayULongArgs { value: args }).as_union_value()
        }
        Value::Float => {
            let args = to_args::<f32>(fbb, value)?;
            ArrayFloat::create(fbb, &ArrayFloatArgs { value: args }).as_union_value()
        }
        Value::Double => {
            let args = to_args::<f64>(fbb, value)?;
            ArrayDouble::create(fbb, &ArrayDoubleArgs { value: args }).as_union_value()
        }
        Value::ArrayByte => {
            let args = to_args::<i8>(fbb, value)?;
            ArrayByte::create(fbb, &ArrayByteArgs { value: args }).as_union_value()
        }
        Value::ArrayShort => {
            let args = to_args::<i16>(fbb, value)?;
            ArrayShort::create(fbb, &ArrayShortArgs { value: args }).as_union_value()
        }
        Value::ArrayInt => {
            let args = to_args::<i32>(fbb, value)?;
            ArrayInt::create(fbb, &ArrayIntArgs { value: args }).as_union_value()
        }
        Value::ArrayLong => {
            let args = to_args::<i64>(fbb, value)?;
            ArrayLong::create(fbb, &ArrayLongArgs { value: args }).as_union_value()
        }
        Value::ArrayUByte => {
            let args = to_args::<u8>(fbb, value)?;
            ArrayUByte::create(fbb, &ArrayUByteArgs { value: args }).as_union_value()
        }
        Value::ArrayUShort => {
            let args = to_args::<u16>(fbb, value)?;
            ArrayUShort::create(fbb, &ArrayUShortArgs { value: args }).as_union_value()
        }
        Value::ArrayUInt => {
            let args = to_args::<u32>(fbb, value)?;
            ArrayUInt::create(fbb, &ArrayUIntArgs { value: args }).as_union_value()
        }
        Value::ArrayULong => {
            let args = to_args::<u64>(fbb, value)?;
            ArrayULong::create(fbb, &ArrayULongArgs { value: args }).as_union_value()
        }
        Value::ArrayFloat => {
            let args = to_args::<f32>(fbb, value)?;
            ArrayFloat::create(fbb, &ArrayFloatArgs { value: args }).as_union_value()
        }
        Value::ArrayDouble => {
            let args = to_args::<f64>(fbb, value)?;
            ArrayDouble::create(fbb, &ArrayDoubleArgs { value: args }).as_union_value()
        }
        _ => unreachable!(),
    })
}
