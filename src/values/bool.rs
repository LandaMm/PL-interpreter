use serde::Serialize;

use crate::{RuntimeValue, ValueType};

#[derive(Debug, Clone, Serialize)]
pub struct BoolValue {
    kind: ValueType,
    value: bool,
}

impl RuntimeValue for BoolValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl From<bool> for BoolValue {
    fn from(value: bool) -> Self {
        Self {
            kind: ValueType::Boolean,
            value,
        }
    }
}

impl BoolValue {
    pub fn value(&self) -> bool {
        self.value
    }
}
