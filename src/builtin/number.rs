use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    cast_value, ClosureType, DecimalValue, IntegerValue, Key, NullValue, ObjectValue, RuntimeValue,
    Value, ValueType,
};

use super::{mk_native_fn, mk_runtime_value};

pub fn abs(value: Box<dyn RuntimeValue>) -> ClosureType {
    Arc::new(Mutex::new(
        move |_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            // TODO: do we want to return null for non-empty arguments
            // if !args.is_empty() {
            //     return mk_runtime_value(Box::new(NullValue::default()));
            // }

            match value.kind() {
                ValueType::Decimal => {
                    let val = cast_value::<DecimalValue>(&value).unwrap();
                    mk_runtime_value(Box::new(DecimalValue::from(val.value().abs())))
                }
                ValueType::Integer => {
                    let val = cast_value::<IntegerValue>(&value).unwrap();
                    mk_runtime_value(Box::new(IntegerValue::from(val.value().abs())))
                }
                _ => mk_runtime_value(Box::new(NullValue::default())),
            }
        },
    ))
}

pub fn get_number_object(number_value: &Box<dyn RuntimeValue>) -> Box<ObjectValue> {
    let mut map: HashMap<Key, Value> = HashMap::new();

    let value = dyn_clone::clone_box(&**number_value);

    map.insert(
        "abs".to_string(),
        mk_native_fn("number.abs".to_string(), abs(value)),
    );

    Box::new(ObjectValue::from(map))
}
