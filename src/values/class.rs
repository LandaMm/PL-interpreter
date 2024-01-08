use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use pl_ast::Node;
use serde::Serialize;

use super::{RuntimeValue, ValueType};

#[derive(Debug, Serialize)]
pub struct ClassProperty {
    pub name: String,
    pub value: Arc<Mutex<Box<dyn RuntimeValue>>>,
    pub is_static: bool,
}

impl Clone for ClassProperty {
    fn clone(&self) -> Self {
        let value = self.value.lock().expect("class_clone: failed to get value");
        Self {
            name: self.name.clone(),
            value: Arc::new(Mutex::new(dyn_clone::clone_box(&**value))),
            is_static: self.is_static,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ClassMethodParameter {
    pub name: String,
    pub default_value: Option<Arc<Mutex<Box<dyn RuntimeValue>>>>,
}

impl Clone for ClassMethodParameter {
    fn clone(&self) -> Self {
        let default_value: Option<Arc<Mutex<Box<dyn RuntimeValue>>>> = match &self.default_value {
            Some(value) => {
                let value = value
                    .lock()
                    .expect("meth_param_clone: failed to get default value");
                Some(Arc::new(Mutex::new(dyn_clone::clone_box(&**value))))
            }
            None => None,
        };
        Self {
            name: self.name.clone(),
            default_value,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct ClassMethod {
    pub name: String,
    pub args: Vec<ClassMethodParameter>,
    pub body: Box<Node>,
    pub is_static: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ClassValue {
    kind: ValueType,
    pub name: String,
    pub super_class: Option<Box<ClassValue>>,
    pub properties: Vec<ClassProperty>,
    pub methods: HashMap<String, ClassMethod>,
}

impl RuntimeValue for ClassValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}

impl Default for ClassValue {
    fn default() -> Self {
        Self {
            kind: ValueType::Class,
            name: "".to_string(),
            super_class: None,
            properties: Vec::new(),
            methods: HashMap::new(),
        }
    }
}

impl ClassValue {
    pub fn insert_property(&mut self, property: ClassProperty) {
        if let Some(index) = self.properties.iter().position(|x| x.name == property.name) {
            self.properties.remove(index);
        }
        self.properties.push(property);
    }

    pub fn insert_method(&mut self, method: ClassMethod) {
        self.methods.insert(method.name.clone(), method);
    }

    pub fn copy_properties(&mut self, target_class: &ClassValue) {
        for property in target_class
            .properties
            .iter()
            .filter(|prop| !prop.is_static)
        {
            self.insert_property(property.clone());
        }
    }

    pub fn copy_methods(&mut self, target_class: &ClassValue) {
        for (_, method) in target_class
            .methods
            .iter()
            .filter(|method| method.0 != "__new__" && !method.1.is_static)
        {
            self.insert_method(method.clone());
        }
    }

    pub fn get_static_property(&self, property_name: String) -> Option<ClassProperty> {
        self.properties
            .iter()
            .find(|prop| prop.name == property_name && prop.is_static)
            .cloned()
    }

    pub fn get_static_method(&self, method_name: String) -> Option<ClassMethod> {
        let method = self
            .methods
            .iter()
            .find(|prop| prop.0.clone() == method_name && prop.1.is_static);
        if method.is_none() {
            return None;
        }

        let method = method.unwrap();

        Some(method.1.clone())
    }
}
