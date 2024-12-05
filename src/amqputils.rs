use lapin::types::{AMQPType, AMQPValue};

pub(crate) fn get_string_value(value: &AMQPValue) -> Option<String> {
    match value.get_type() {
        AMQPType::Boolean => {
            Some(value.as_bool().unwrap().to_string())
        }
        AMQPType::ShortShortInt => {
            Some(value.as_short_short_int().unwrap().to_string())
        }
        AMQPType::ShortShortUInt => {
            Some(value.as_short_short_uint().unwrap().to_string())
        }
        AMQPType::ShortInt => {
            Some(value.as_short_int().unwrap().to_string())
        }
        AMQPType::ShortUInt => {
            Some(value.as_short_uint().unwrap().to_string())
        }
        AMQPType::LongInt => {
            Some(value.as_long_int().unwrap().to_string())
        }
        AMQPType::LongUInt => {
            Some(value.as_long_uint().unwrap().to_string())
        }
        AMQPType::LongLongInt => {
            Some(value.as_long_long_int().unwrap().to_string())
        }
        AMQPType::LongLongUInt => {
            Some(value.as_long_long_int().unwrap().to_string())
        }
        AMQPType::Float => {
            Some(value.as_float().unwrap().to_string())
        }
        AMQPType::Double => {
            Some(value.as_double().unwrap().to_string())
        }
        AMQPType::DecimalValue => {
            Some(value.as_decimal_value().unwrap().value.to_string())
        }
        AMQPType::ShortString => {
            Some(value.as_short_string().unwrap().to_string())
        }
        AMQPType::LongString => {
            Some(value.as_long_string().unwrap().to_string())
        }
        AMQPType::FieldArray => {
            None
        }
        AMQPType::Timestamp => {
            Some(value.as_timestamp().unwrap().to_string())
        }
        AMQPType::FieldTable => {
            None
        }
        AMQPType::ByteArray => {
            None
        }
        AMQPType::Void => {
            None
        }
    }
}
