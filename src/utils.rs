use crate::tasks::Value;
use kdl::KdlValue;

#[macro_export]
macro_rules! S {
    ($l:expr) => {
        String::from($l)
    };
}

pub fn kdl_value_to_value(value: &KdlValue) -> Value {
    match value {
        KdlValue::String(s) => Value::String(s.clone()),
        KdlValue::Integer(i) => {
            let val: i64 = i.clone().try_into().unwrap();
            Value::Int(val)
        }
        KdlValue::Float(f) => Value::Float(f.clone()),
        KdlValue::Bool(b) => Value::Bool(*b),
        KdlValue::Null => Value::Null,
    }
}
