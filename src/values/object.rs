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

    pub fn get_property(&self, key: Key) -> Option<Value> {
        if let Some(property) = self.map.get(&key) {
            let value = property
                .lock()
                .expect("object.get_property(): failed to get property value");
            return Some(Arc::new(Mutex::new(dyn_clone::clone_box(&**value))));
        }
        None
    }

    pub fn assign_property(&mut self, key: Key, value: Value) {
        self.map.entry(key).and_modify(|val| *val = value);
    }
}
