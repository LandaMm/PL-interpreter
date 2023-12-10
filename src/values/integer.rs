use super::{RuntimeValue, ValueType};

#[derive(Debug, Clone, Copy)]
pub struct IntegerValue {
    kind: ValueType,
    value: isize,
}

impl RuntimeValue for IntegerValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}

impl From<isize> for IntegerValue {
    fn from(value: isize) -> Self {
        Self {
            kind: ValueType::Integer,
            value,
        }
    }
}

impl IntegerValue {
    pub fn value(&self) -> isize {
        self.value
    }
}
