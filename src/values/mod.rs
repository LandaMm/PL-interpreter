use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;

mod bool;
mod decimal;
mod integer;
mod native_fn;
mod null;

pub use bool::*;
pub use decimal::*;
pub use integer::*;
pub use native_fn::*;
pub use null::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ValueType {
    Null,
    Integer,
    Decimal,
    Boolean,
    NativeFn,
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
            ValueType::Boolean => {
                let clo = dyn_clone::clone_box(self);
                let any = clo.into_any();
                let value = any.downcast::<BoolValue>().unwrap();
                write!(f, "{:?}", value)
            }
            ValueType::NativeFn => {
                let clo = dyn_clone::clone_box(self);
                let any = clo.into_any();
                let value = any.downcast::<NativeFnValue>().unwrap();
                write!(f, "{:?}", value)
            }
        }
    }
}
