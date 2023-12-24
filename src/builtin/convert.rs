use std::sync::{Arc, Mutex};

use crate::{cast_value, ArrayValue, NullValue, ObjectValue, RuntimeValue, StringValue, ValueType};

use super::{mk_runtime_value, stringify};

pub fn native_string_convert(
    args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args
        .get(0)
        .unwrap()
        .lock()
        .expect("string_converter: failed to get first argument");

    let value = match arg.kind() {
        ValueType::Array => {
            let arr = cast_value::<ArrayValue>(&arg).unwrap();
            format!("<array ({} items)>", arr.value().len())
        }
        ValueType::Object => {
            let obj = cast_value::<ObjectValue>(&arg).unwrap();
            format!("<object ({} pairs)>", obj.map().len())
        }
        _ => stringify(dyn_clone::clone_box(&**arg)),
    };

    mk_runtime_value(Box::new(StringValue::from(value)))
}
