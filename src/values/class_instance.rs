use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{RuntimeValue, ValueType};

pub type ClassInstanceMapValue = Arc<Mutex<Box<dyn RuntimeValue>>>;

#[derive(Debug, Clone)]
pub struct ClassInstanceValue {
    kind: ValueType,
    class_name: String,
    map: HashMap<String, ClassInstanceMapValue>,
}

impl RuntimeValue for ClassInstanceValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl From<(String, HashMap<String, ClassInstanceMapValue>)> for ClassInstanceValue {
    fn from((class_name, map): (String, HashMap<String, ClassInstanceMapValue>)) -> Self {
        Self {
            kind: ValueType::Object,
            class_name,
            map,
        }
    }
}

impl ClassInstanceValue {
    pub fn map(&self) -> HashMap<String, ClassInstanceMapValue> {
        self.map.clone()
    }

    pub fn class_name(&self) -> String {
        self.class_name.clone()
    }
}
