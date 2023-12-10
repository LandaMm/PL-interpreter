use super::{RuntimeValue, ValueType};

#[derive(Debug, Clone)]
pub struct NullValue {
    kind: ValueType,
}

impl RuntimeValue for NullValue {
    fn kind(&self) -> ValueType {
        self.kind
    }
}

impl Default for NullValue {
    fn default() -> Self {
        Self {
            kind: ValueType::Null,
        }
    }
}
