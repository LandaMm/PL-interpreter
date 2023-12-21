use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use pl_ast::Node;

use super::{RuntimeValue, ValueType};

#[derive(Debug)]
pub struct ClassProperty {
    pub name: String,
    pub value: Arc<Mutex<Box<dyn RuntimeValue>>>,
    pub is_static: bool,
}

impl Clone for ClassProperty {
    fn clone(&self) -> Self {
        let value = self.value.lock().unwrap();
        Self {
            name: self.name.clone(),
            value: Arc::new(Mutex::new(dyn_clone::clone_box(&**value))),
            is_static: self.is_static,
        }
    }
}

#[derive(Debug)]
pub struct ClassMethodParameter {
    pub name: String,
    pub default_value: Option<Arc<Mutex<Box<dyn RuntimeValue>>>>,
}

impl Clone for ClassMethodParameter {
    fn clone(&self) -> Self {
        let default_value: Option<Arc<Mutex<Box<dyn RuntimeValue>>>> = match &self.default_value {
            Some(value) => {
                let value = value.lock().unwrap();
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

#[derive(Clone, Debug)]
pub struct ClassMethod {
    pub name: String,
    pub args: Vec<ClassMethodParameter>,
    pub body: Box<Node>,
    pub is_static: bool,
}

#[derive(Debug, Clone)]
pub struct ClassValue {
    kind: ValueType,
    pub name: String,
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
        for property in &target_class.properties {
            self.insert_property(property.clone());
        }
    }

    pub fn copy_methods(&mut self, target_class: &ClassValue) {
        for (_, method) in &target_class.methods {
            self.insert_method(method.clone());
        }
    }
}
