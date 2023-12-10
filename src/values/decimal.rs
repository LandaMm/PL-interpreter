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

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
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
    pub fn value(&self) -> f64 {
        self.value
    }
}
