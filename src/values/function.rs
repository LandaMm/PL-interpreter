use std::{
    fmt::{self, Debug, Formatter},
    sync::{Arc, Mutex},
};

use pl_ast::Node;
use serde::Serialize;

use crate::{EnvironmentId, RuntimeValue, ValueType};

#[derive(Clone, Serialize)]
pub struct FunctionParameter {
    pub name: String,
    pub default_value: Option<Arc<Mutex<Box<dyn RuntimeValue>>>>,
}

impl Debug for FunctionParameter {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("FunctionParameter")
            .field("name", &self.name)
            .field("default_value", &self.default_value)
            .finish()
    }
}

impl FunctionParameter {
    pub fn new(name: String, default_value: Option<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Self {
        Self {
            name,
            default_value,
        }
    }
}

// impl Clone for FunctionParameter {
//     fn clone(&self) -> Self {
//         Self {
//             name: self.name.clone(),
//             default_value: match &self.default_value {
//                 None => None,
//                 Some(value) => Some(dyn_clone::clone_box(&**value)),
//             },
//         }
//     }
// }

#[derive(Debug, Clone, Serialize)]
pub struct FunctionValue {
    kind: ValueType,
    pub name: String,
    pub parameters: Vec<FunctionParameter>,
    pub declaration_env: EnvironmentId,
    pub body: Box<Node>,
}

impl FunctionValue {
    pub fn new(
        name: String,
        parameters: Vec<FunctionParameter>,
        declaration_env: EnvironmentId,
        body: Box<Node>,
    ) -> Self {
        Self {
            kind: ValueType::Function,
            name,
            parameters,
            declaration_env,
            body,
        }
    }
}

impl RuntimeValue for FunctionValue {
    fn kind(&self) -> ValueType {
        self.kind
    }

    fn into_any(&self) -> Box<dyn std::any::Any> {
        Box::new(dyn_clone::clone(self))
    }
}
