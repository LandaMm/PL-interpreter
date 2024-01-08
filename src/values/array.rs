use std::sync::{Arc, Mutex};

use serde::Serialize;

use super::{RuntimeValue, ValueType};

#[derive(Debug, Serialize)]
pub struct ArrayValue {
    kind: ValueType,
    value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
}

impl Clone for ArrayValue {
    fn clone(&self) -> Self {
        let mut cloned_value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = vec![];
        for value in self.value.as_slice() {
            cloned_value.push(Arc::new(Mutex::new(dyn_clone::clone_box(
                &**value
                    .lock()
                    .expect("array_clone: failed to get value of slice"),
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
            value,
        }
    }
}

impl ArrayValue {
    pub fn value(&self) -> Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> {
        let mut cloned_value: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = vec![];
        for value in self.value.as_slice() {
            cloned_value.push(Arc::new(Mutex::new(dyn_clone::clone_box(
                &**value
                    .lock()
                    .expect("array.value(): failed to get value of slice"),
            ))));
        }
        cloned_value
    }

    pub fn append_element(&mut self, element: Arc<Mutex<Box<dyn RuntimeValue>>>) {
        self.value.push(element)
    }
}
