use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    cast_value, ClosureType, IntegerValue, Key, NullValue, ObjectValue, RuntimeValue, StringValue,
    Value, ValueType,
};

use super::{mk_native_fn, mk_runtime_value};

pub fn char_at(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.len() < 1 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let arg = args.get(0).unwrap().lock().unwrap();

            if arg.kind() != ValueType::Integer {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let index = cast_value::<IntegerValue>(&arg).unwrap();

            if index.value() < 0 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            if value.value().chars().count() <= index.value() as usize {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let char = value.value().chars().nth(index.value() as usize).unwrap();

            mk_runtime_value(Box::new(StringValue::from(char.to_string())))
        },
    ))
}

pub fn get_string_object(string_value: &StringValue) -> Box<ObjectValue> {
    let mut map: HashMap<Key, Value> = HashMap::new();

    map.insert(
        "char_at".to_string(),
        mk_native_fn("string.char_at".to_string(), char_at(string_value.clone())),
    );

    Box::new(ObjectValue::from(map))
}
