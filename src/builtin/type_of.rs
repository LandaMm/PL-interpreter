use std::sync::{Arc, Mutex};

use crate::{NullValue, RuntimeValue, StringValue, ValueType};

use super::mk_runtime_value;

pub fn native_type_of(
    args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args.get(0).unwrap().lock().unwrap();

    let value_type: String = match arg.kind() {
        ValueType::Array => "array".into(),
        ValueType::Boolean => "boolean".into(),
        ValueType::Decimal | ValueType::Integer => "number".into(),
        ValueType::Function | ValueType::NativeFn => "function".into(),
        ValueType::Null => "null".into(),
        ValueType::Object => "object".into(),
        ValueType::String => "string".into(),
        // TODO: should we return "object" here?
        ValueType::Class => "class".into(),
    };

    mk_runtime_value(Box::new(StringValue::from(value_type)))
}
