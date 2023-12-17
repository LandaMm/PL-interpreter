use std::sync::{Arc, Mutex};

use super::{RuntimeValue, ValueType};

#[derive(Debug)]
pub struct ArrayValue {
    kind: ValueType,
    value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
}

impl Clone for ArrayValue {
    fn clone(&self) -> Self {
        let mut cloned_value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = vec![];
        for value in self.value.as_slice() {
            cloned_value.push(Arc::new(Mutex::new(dyn_clone::clone_box(
                &**value.lock().unwrap(),
            ))));
        }
        Self {
            kind: self.kind.clone(),
            value: cloned_value,
        }
    }
}

impl RuntimeValue for ArrayValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl From<Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>> for ArrayValue {
    fn from(value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Self {
        Self {
            kind: ValueType::Array,
            value: value,
        }
    }
}

impl ArrayValue {
    pub fn value(&self) -> Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> {
        let mut cloned_value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = vec![];
        for value in self.value.as_slice() {
            cloned_value.push(Arc::new(Mutex::new(dyn_clone::clone_box(
                &**value.lock().unwrap(),
            ))));
        }
        cloned_value
    }
}
