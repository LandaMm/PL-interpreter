use std::{
    collections::HashMap,
    fmt::{Debug, Formatter},
};

use crate::{
    ArrayValue, BoolValue, ClassInstanceValue, ClassValue, DecimalValue, FunctionValue,
    IntegerValue, NativeFnValue, ObjectValue, RuntimeValue, StringValue, ValueType,
};

use super::cast_value;

const ARRAY_MAX_ITEMS: usize = 10;
const OBJECT_MAX_ITEMS: usize = 30;

struct DebugHashMap<'a>(&'a HashMap<String, String>, isize);

impl<'a> Debug for DebugHashMap<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut debug_map = f.debug_map();
        for (key, value) in self.0 {
            debug_map.entry(&key, &DebugValue(&value));
        }
        if self.1 > 0 {
            debug_map.entry(
                &"...".to_string(),
                &DebugValue(&format!("more {} fields", self.1)),
            );
        }
        debug_map.finish()
    }
}

struct DebugValue<'a>(&'a str);

impl<'a> Debug for DebugValue<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

pub fn stringify(value: Box<dyn RuntimeValue>) -> String {
    match value.kind() {
        ValueType::Null => "null".to_string(),
        ValueType::Boolean => {
            let boolean = cast_value::<BoolValue>(&value).unwrap();
            boolean.value().to_string()
        }
        ValueType::Decimal => cast_value::<DecimalValue>(&value)
            .unwrap()
            .value()
            .to_string(),
        ValueType::Integer => cast_value::<IntegerValue>(&value)
            .unwrap()
            .value()
            .to_string(),
        ValueType::Function => {
            let function = cast_value::<FunctionValue>(&value).unwrap();
            format!(
                "<function {}({})>",
                function.name,
                function
                    .parameters
                    .clone()
                    .into_iter()
                    .map(|x| x.name)
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
        ValueType::NativeFn => {
            let function = cast_value::<NativeFnValue>(&value).unwrap();
            format!("<native-function {}>", function.name,)
        }
        ValueType::String => {
            let string = cast_value::<StringValue>(&value).unwrap();
            format!("{}", string.value())
        }
        ValueType::Array => {
            let array = cast_value::<ArrayValue>(&value).unwrap();
            format!(
                "[{}{}]",
                array
                    .value()
                    .into_iter()
                    .map(|x| {
                        if x.lock()
                            .expect("stringify: failed to get array item")
                            .kind()
                            == ValueType::String
                        {
                            format!(
                                "\"{}\"",
                                stringify(dyn_clone::clone_box(
                                    &**x.lock()
                                        .expect("stringify: failed to get string array item")
                                ))
                            )
                        } else {
                            stringify(dyn_clone::clone_box(
                                &**x.lock()
                                    .expect("stringify: failed to stringify other type array item"),
                            ))
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", "),
                if array.value().len() > ARRAY_MAX_ITEMS {
                    format!(", ...more {} items", array.value().len() - ARRAY_MAX_ITEMS)
                } else {
                    "".to_string()
                }
            )
        }
        ValueType::Object => {
            let object = cast_value::<ObjectValue>(&value).unwrap();
            let mut map: HashMap<String, String> = HashMap::new();
            for (key, value) in object.map().iter().take(OBJECT_MAX_ITEMS) {
                map.insert(
                    key.clone(),
                    stringify(dyn_clone::clone_box(
                        &**value.lock().expect("stringify: failed to get object value"),
                    )),
                );
            }

            format!(
                "{:#?}",
                DebugHashMap(
                    &map,
                    object.map().len() as isize - OBJECT_MAX_ITEMS as isize
                )
            )
        }
        ValueType::Class => {
            let class = cast_value::<ClassValue>(&value).unwrap();
            format!("<class {}>", class.name)
        }
        ValueType::ClassInstance => {
            let class = cast_value::<ClassInstanceValue>(&value).unwrap();
            format!("<class_instance of {}>", class.class_name())
        }
    }
}
