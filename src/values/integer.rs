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
    fn value(&self) -> isize {
        self.value
    }
}
