use std::{cell::RefCell, fmt, rc::Rc};

use crate::{Environment, RuntimeValue, ValueType};

pub struct NativeFnValue {
    kind: ValueType,
    pub call:
        Rc<RefCell<dyn Fn(Vec<Box<dyn RuntimeValue>>, &Environment) -> Box<dyn RuntimeValue>>>,
}

impl NativeFnValue {
    pub fn new(
        call: impl Fn(Vec<Box<dyn RuntimeValue>>, &Environment) -> Box<dyn RuntimeValue> + 'static,
    ) -> Self {
        Self {
            kind: ValueType::NativeFn,
            call: Rc::new(RefCell::new(call)),
        }
    }
}

impl Clone for NativeFnValue {
    fn clone(&self) -> Self {
        Self {
            kind: self.kind.clone(),
            call: self.call.clone(),
        }
    }
}

impl fmt::Debug for NativeFnValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeFnValue")
            .field("kind", &self.kind)
            .finish()
    }
}

impl RuntimeValue for NativeFnValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(self: Box<Self>) -> Box<dyn std::any::Any> {
        self
    }
}
