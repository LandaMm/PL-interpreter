use crate::{RuntimeValue, ValueType};

#[derive(Debug, Clone)]
pub struct BoolValue {
    kind: ValueType,
    value: bool,
}

impl RuntimeValue for BoolValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
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
