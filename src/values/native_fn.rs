use std::{
    fmt,
    sync::{Arc, Mutex},
};

use serde::{ser::SerializeStruct, Serialize, Serializer};

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
        (self.fc.lock().expect("with_fn_call: failed to get fc"))(args)
    }
}

#[derive(Clone)]
pub struct NativeFnValue {
    pub name: String,
    kind: ValueType,
    call: WithFnCall<ClosureType>,
}

impl NativeFnValue {
    pub fn new(name: String, call: WithFnCall<ClosureType>) -> Self {
        Self {
            kind: ValueType::NativeFn,
            name,
            call,
        }
    }

    pub fn callee(&self) -> WithFnCall<ClosureType> {
        self.call.clone()
    }
}

impl Serialize for NativeFnValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("NativeFn", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("kind", &self.kind())?;
        state.end()
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
