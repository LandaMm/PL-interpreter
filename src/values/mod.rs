use std::{any::Any, fmt::Debug};

use dyn_clone::DynClone;

use erased_serde::serialize_trait_object;

mod array;
mod bool;
mod class;
mod class_instance;
mod decimal;
mod function;
mod integer;
mod native_fn;
mod null;
mod object;
mod string;

pub use array::*;
pub use bool::*;
pub use class::*;
pub use class_instance::*;
pub use decimal::*;
pub use function::*;
pub use integer::*;
pub use native_fn::*;
pub use null::*;
pub use object::*;
use serde::Serialize;
pub use string::*;

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum ValueType {
    Null,
    Integer,
    Decimal,
    Boolean,
    NativeFn,
    Function,
    String,
    Array,
    Object,
    Class,
    ClassInstance,
}

pub trait RuntimeValue: DynClone + Debug + Send + Sync + erased_serde::Serialize {
    fn kind(&self) -> ValueType;
    #[allow(clippy::wrong_self_convention)]
    fn into_any(&self) -> Box<dyn Any>;
}

serialize_trait_object!(RuntimeValue);
