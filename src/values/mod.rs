use dyn_clone::DynClone;

mod decimal;
mod integer;
mod null;

#[derive(Debug, Clone, Copy)]
pub enum ValueType {
    Null,
    Integer,
    Decimal,
}

pub trait RuntimeValue: DynClone {
    fn kind(&self) -> ValueType;
}
