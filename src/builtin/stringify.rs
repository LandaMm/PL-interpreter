use crate::{
    BoolValue, DecimalValue, FunctionValue, IntegerValue, NativeFnValue, RuntimeValue, ValueType,
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
    }
}
