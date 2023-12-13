use std::{
    fmt,
    sync::{Arc, Mutex},
};

use crate::{RuntimeValue, ValueType};

pub type ClosureType = Arc<
    Mutex<
        dyn Fn(Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>>
            + Send
            + Sync,
    >,
>;

#[derive(Clone)]
pub struct WithFnCall<T> {
    fc: T,
}

impl WithFnCall<ClosureType> {
    pub fn new(fc: ClosureType) -> Self {
        Self { fc }
    }

    pub fn run(
        &self,
        args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
    ) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
        (self.fc.lock().unwrap())(args)
    }
}

#[derive(Clone)]
pub struct NativeFnValue {
    kind: ValueType,
    call: WithFnCall<ClosureType>,
}

impl NativeFnValue {
    pub fn new(call: WithFnCall<ClosureType>) -> Self {
        Self {
            kind: ValueType::NativeFn,
            call,
        }
    }

    pub fn callee(&self) -> WithFnCall<ClosureType> {
        self.call.clone()
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

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}
