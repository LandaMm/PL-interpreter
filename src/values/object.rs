use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{RuntimeValue, ValueType};

pub type Key = String;
pub type Value = Arc<Mutex<Box<dyn RuntimeValue>>>;

#[derive(Debug, Clone)]
pub struct ObjectValue {
    kind: ValueType,
    map: HashMap<Key, Value>,
}

impl RuntimeValue for ObjectValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl From<HashMap<Key, Value>> for ObjectValue {
    fn from(map: HashMap<Key, Value>) -> Self {
        Self {
            kind: ValueType::Object,
            map,
        }
    }
}

impl ObjectValue {
    pub fn map(&self) -> HashMap<Key, Value> {
        self.map.clone()
    }

    pub fn assign_property(&mut self, key: Key, value: Value) {
        self.map.entry(key).and_modify(|val| *val = value);
    }
}
