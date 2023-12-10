use super::{RuntimeValue, ValueType};

#[derive(Debug, Clone)]
pub struct NullValue {
    kind: ValueType,
}

impl RuntimeValue for NullValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl Default for NullValue {
    fn default() -> Self {
        Self {
            kind: ValueType::Null,
        }
    }
}
