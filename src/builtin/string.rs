use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    cast_value, convert_to_string, ArrayValue, ClosureType, IntegerValue, Key, NullValue,
    ObjectValue, RuntimeValue, StringValue, Value, ValueType,
};

use super::{mk_native_fn, mk_runtime_value};

pub fn get_char(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.len() < 1 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let arg = args
                .get(0)
                .unwrap()
                .lock()
                .expect("string.get_char: failed to get argument");

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

pub fn concat(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            let mut result = String::from(value.value());
            for arg in args {
                let value = arg.lock().expect("string.concat: failed to get argument");
                if value.kind() != ValueType::String {
                    // TODO: raise type error
                    continue;
                }
                let str_value = cast_value::<StringValue>(&value).unwrap();
                result.push_str(&str_value.value());
            }

            mk_runtime_value(Box::new(StringValue::from(result)))
        },
    ))
}

pub fn substr(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.len() < 1 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let skip_val = args
                .get(0)
                .unwrap()
                .lock()
                .expect("string.substr: failed to get argument");

            if skip_val.kind() != ValueType::Integer {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let skip = cast_value::<IntegerValue>(&skip_val).unwrap();

            if skip.value() < 0 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let take_val = if args.len() < 2 {
                Box::new(IntegerValue::from(
                    value.value().chars().count() as isize - skip.value() as isize,
                ))
            } else {
                dyn_clone::clone_box(
                    &**args
                        .get(1)
                        .unwrap()
                        .lock()
                        .expect("string.substr: failed to get second argument"),
                )
            };

            if take_val.kind() != ValueType::Integer {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let mut take = cast_value::<IntegerValue>(&take_val).unwrap();

            if take.value() < 0 {
                if take.value().abs() >= value.value().chars().count() as isize - skip.value() {
                    return mk_runtime_value(Box::new(NullValue::default()));
                }
                take = Box::new(IntegerValue::from(
                    value.value().chars().count() as isize - skip.value() + take.value(),
                ))
            }

            let result: String = value
                .value()
                .chars()
                .skip(skip.value() as usize)
                .take(take.value() as usize)
                .collect();

            mk_runtime_value(Box::new(StringValue::from(result)))
        },
    ))
}

pub fn upper(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            mk_runtime_value(Box::new(StringValue::from(value.value().to_uppercase())))
        },
    ))
}

pub fn lower(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            mk_runtime_value(Box::new(StringValue::from(value.value().to_lowercase())))
        },
    ))
}

pub fn trim(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            mk_runtime_value(Box::new(StringValue::from(
                value.value().trim().to_string(),
            )))
        },
    ))
}

pub fn trim_start(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            mk_runtime_value(Box::new(StringValue::from(
                value.value().trim_start().to_string(),
            )))
        },
    ))
}

pub fn trim_end(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            mk_runtime_value(Box::new(StringValue::from(
                value.value().trim_end().to_string(),
            )))
        },
    ))
}

pub fn replace(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.len() != 2 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let search_val = args
                .get(0)
                .unwrap()
                .lock()
                .expect("string.replace: failed to get argument");

            if search_val.kind() != ValueType::String {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let replace_val = args
                .get(1)
                .unwrap()
                .lock()
                .expect("string.replace: failed to get second argument");

            if replace_val.kind() != ValueType::String {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let search = cast_value::<StringValue>(&search_val).unwrap();
            let replace = cast_value::<StringValue>(&replace_val).unwrap();

            mk_runtime_value(Box::new(StringValue::from(
                value.value().replace(&search.value(), &replace.value()),
            )))
        },
    ))
}

pub fn split(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.len() != 1 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let split_val = args
                .get(0)
                .unwrap()
                .lock()
                .expect("string.split: failed to get argument");

            if split_val.kind() != ValueType::String {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let split = cast_value::<StringValue>(&split_val).unwrap();

            let parts = value
                .value()
                .split(&split.value())
                .map(|x| x.to_string())
                .collect::<Vec<String>>();

            let arr: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = parts
                .into_iter()
                .map(|x| mk_runtime_value(Box::new(StringValue::from(x))))
                .collect();

            mk_runtime_value(Box::new(ArrayValue::from(arr)))
        },
    ))
}

pub fn join(value: StringValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.len() != 1 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let join_val = args
                .get(0)
                .unwrap()
                .lock()
                .expect("string.join: failed to get argument");

            if join_val.kind() != ValueType::Array {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let array = cast_value::<ArrayValue>(&join_val).unwrap();

            let joined = array
                .value()
                .iter()
                .map(|item| {
                    convert_to_string(&item.lock().expect("string.join: failed to get array item"))
                })
                .collect::<Vec<String>>()
                .join(&value.value());

            mk_runtime_value(Box::new(StringValue::from(joined)))
        },
    ))
}

pub fn get_string_object(string_value: &StringValue) -> Box<ObjectValue> {
    let mut map: HashMap<Key, Value> = HashMap::new();

    map.insert(
        "get".to_string(),
        mk_native_fn("string.get".to_string(), get_char(string_value.clone())),
    );

    map.insert(
        "concat".to_string(),
        mk_native_fn("string.concat".to_string(), concat(string_value.clone())),
    );

    map.insert(
        "substr".to_string(),
        mk_native_fn("string.substr".to_string(), substr(string_value.clone())),
    );

    map.insert(
        "upper".to_string(),
        mk_native_fn("string.upper".to_string(), upper(string_value.clone())),
    );

    map.insert(
        "lower".to_string(),
        mk_native_fn("string.lower".to_string(), lower(string_value.clone())),
    );

    map.insert(
        "replace".to_string(),
        mk_native_fn("string.replace".to_string(), replace(string_value.clone())),
    );

    map.insert(
        "trim".to_string(),
        mk_native_fn("string.trim".to_string(), trim(string_value.clone())),
    );

    map.insert(
        "trim_start".to_string(),
        mk_native_fn(
            "string.trim_start".to_string(),
            trim_start(string_value.clone()),
        ),
    );

    map.insert(
        "trim_end".to_string(),
        mk_native_fn(
            "string.trim_end".to_string(),
            trim_end(string_value.clone()),
        ),
    );

    map.insert(
        "split".to_string(),
        mk_native_fn("string.split".to_string(), split(string_value.clone())),
    );

    map.insert(
        "join".to_string(),
        mk_native_fn("string.join".to_string(), join(string_value.clone())),
    );

    map.insert(
        "length".to_string(),
        mk_runtime_value(Box::new(IntegerValue::from(
            string_value.value().chars().count() as isize,
        ))),
    );

    Box::new(ObjectValue::from(map))
}
