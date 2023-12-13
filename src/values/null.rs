use super::{RuntimeValue, ValueType};

#[derive(Debug, Clone)]
pub struct NullValue {
    kind: ValueType,
}

impl RuntimeValue for NullValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl Default for NullValue {
    fn default() -> Self {
        Self {
            kind: ValueType::Null,
        }
    }
}
