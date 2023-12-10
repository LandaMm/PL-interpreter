use super::{RuntimeValue, ValueType};

#[derive(Debug, Clone, Copy)]
pub struct DecimalValue {
    kind: ValueType,
    value: f64,
}

impl RuntimeValue for DecimalValue {
    fn kind(&self) -> ValueType {
        self.kind
    }
}

impl From<f64> for DecimalValue {
    fn from(value: f64) -> Self {
        Self {
            kind: ValueType::Decimal,
            value,
        }
    }
}

impl DecimalValue {
    fn value(&self) -> f64 {
        self.value
    }
}
