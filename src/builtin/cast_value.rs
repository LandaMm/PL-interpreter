use std::any::Any;

use crate::RuntimeValue;

pub fn cast_value<T>(value: &Box<dyn RuntimeValue>) -> Result<Box<T>, Box<dyn Any>>
where
    T: RuntimeValue + 'static,
{
    dyn_clone::clone_box(&value).into_any().downcast::<T>()
}
