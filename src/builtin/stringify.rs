use crate::{
    ArrayValue, BoolValue, DecimalValue, FunctionValue, IntegerValue, NativeFnValue, RuntimeValue,
    StringValue, ValueType,
};

use super::cast_value;

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
                "[{}]",
                array
                    .value()
                    .into_iter()
                    .map(|x| {
                        if x.lock().unwrap().kind() == ValueType::String {
                            format!(
                                "\"{}\"",
                                stringify(dyn_clone::clone_box(&**x.lock().unwrap()))
                            )
                        } else {
                            stringify(dyn_clone::clone_box(&**x.lock().unwrap()))
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }
}
