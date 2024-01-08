use serde::Serialize;

use super::{RuntimeValue, ValueType};

#[derive(Debug, Clone, Serialize)]
pub struct StringValue {
    kind: ValueType,
    value: String,
}

impl RuntimeValue for StringValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl From<String> for StringValue {
    fn from(value: String) -> Self {
        Self {
            kind: ValueType::String,
            value,
        }
    }
}

impl StringValue {
    pub fn value(&self) -> String {
        self.value.clone()
    }
}
