use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;

mod decimal;
mod integer;
mod null;

pub use decimal::*;
pub use integer::*;
pub use null::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValueType {
    Null,
    Integer,
    Decimal,
}

pub trait RuntimeValue: DynClone {
    fn kind(&self) -> ValueType;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl Debug for dyn RuntimeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind() {
            ValueType::Decimal => {
                let clo = dyn_clone::clone_box(self);
                let any = clo.into_any();
                let decimal = any.downcast::<DecimalValue>().unwrap();
                write!(f, "{:?}", decimal)
            }
            ValueType::Integer => {
                let clo = dyn_clone::clone_box(self);
                let any = clo.into_any();
                let value = any.downcast::<IntegerValue>().unwrap();
                write!(f, "{:?}", value)
            }
            ValueType::Null => {
                write!(f, "Null")
            }
        }
    }
}
