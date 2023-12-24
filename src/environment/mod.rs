use core::fmt;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Formatter},
    sync::{Arc, Mutex},
};

use crate::{macros::bail, InterpreterError, RuntimeValue};

pub type EnvironmentId = u64;

#[derive(Debug)]
pub struct ScopeState {
    pub scopes: HashMap<EnvironmentId, Environment>,
    pub last_generated_id: EnvironmentId,
}

impl ScopeState {
    pub fn new() -> Self {
        Self {
            scopes: HashMap::new(),
            last_generated_id: 0,
        }
    }

    pub fn generate_scope_id(&mut self) -> EnvironmentId {
        self.last_generated_id += 1;
        self.last_generated_id
    }

    pub fn get_scope(&self, id: EnvironmentId) -> Option<&Environment> {
        self.scopes.get(&id)
    }

    pub fn get_scope_mut(&mut self, id: EnvironmentId) -> Option<&mut Environment> {
        self.scopes.get_mut(&id)
    }

    pub fn create_environment(&mut self, parent_env: Option<EnvironmentId>) -> EnvironmentId {
        let env = Environment::new(parent_env);
        let env_id = self.append_environment(env);
        env_id
    }

    pub fn append_environment(&mut self, mut environment: Environment) -> EnvironmentId {
        if environment
            .parent
            .is_some_and(|parent_id| self.get_scope(parent_id).is_none())
        {
            panic!("Parent for provided scope is not found: {:?}", environment);
        }

        let id = self.generate_scope_id();

        environment.id = id;

        self.scopes.insert(id, environment);

        id
    }

    pub fn assign_variable(
        &mut self,
        variable_name: String,
        value: Arc<Mutex<Box<dyn RuntimeValue>>>,
        env_id: EnvironmentId,
        ignore_constant: bool,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let env_id = self.resolve_mut(env_id, variable_name.clone())?;
        let scope = self.get_scope_mut(env_id).unwrap();

        if scope.constants.contains(&variable_name) && !ignore_constant {
            bail!(InterpreterError::ReassignConstant(variable_name.clone()))
        }

        scope
            .variables
            .entry(variable_name.clone())
            .and_modify(|val| *val = value);

        let value = Arc::clone(scope.variables.get(&variable_name).unwrap());
        Ok(value)
    }

    pub fn resolve_mut(
        &mut self,
        env_id: EnvironmentId,
        variable_name: String,
    ) -> Result<EnvironmentId, InterpreterError> {
        let scope = self.get_scope(env_id).unwrap();
        if scope.variables.contains_key(&variable_name) {
            return Ok(scope.id);
        }

        if scope.parent.is_none() {
            bail!(InterpreterError::UnresolvedVariable(variable_name))
        }

        // let scope_state = SCOPE_STATE.lock().unwrap();
        let parent_scope = self.get_scope(scope.parent.unwrap()).unwrap();
        let env_id = parent_scope.resolve(variable_name, self)?;
        Ok(env_id)
    }
}

pub struct Environment {
    pub id: EnvironmentId,
    pub parent: Option<EnvironmentId>,
    pub variables: HashMap<String, Arc<Mutex<Box<dyn RuntimeValue>>>>,
    pub constants: HashSet<String>,
}

impl Debug for Environment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Environment")
            .field("parent", &self.parent)
            .field(
                "variables",
                &format_args!("{:#?}", self.variables), // Format keys for simplicity
            )
            .field("constants", &self.constants)
            .finish()
    }
}

impl Environment {
    pub fn new(parent_env: Option<EnvironmentId>) -> Self {
        Self {
            id: 0,
            parent: parent_env,
            variables: HashMap::new(),
            constants: HashSet::new(),
        }
    }

    pub fn declare_variable(
        &mut self,
        variable_name: String,
        value: Arc<Mutex<Box<dyn RuntimeValue>>>,
        is_constant: bool,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        if self.variables.contains_key(&variable_name) {
            bail!(InterpreterError::VariableDeclarationExist(variable_name))
        }

        self.variables.insert(variable_name.clone(), value);

        if is_constant {
            self.constants.insert(variable_name.clone());
        }

        Ok(Arc::clone(self.variables.get(&variable_name).unwrap()))
    }

    // pub fn assign_variable(
    //     &mut self,
    //     variable_name: String,
    //     value: Arc<Mutex<Box<dyn RuntimeValue>>>,
    // ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
    //     let env_id = self.resolve_mut(variable_name.clone())?;
    //     let mut scope_state = SCOPE_STATE.lock().unwrap();
    //     let scope = scope_state.get_scope_mut(env_id).unwrap();

    //     if scope.constants.contains(&variable_name) {
    //         bail!(InterpreterError::ReassignConstant(variable_name.clone()))
    //     }

    //     scope
    //         .variables
    //         .entry(variable_name.clone())
    //         .and_modify(|val| *val = value);

    //     let value = Arc::clone(scope.variables.get(&variable_name).unwrap());
    //     drop(scope_state);
    //     Ok(value)
    // }

    pub fn lookup_variable(
        &self,
        variable_name: String,
        scope_state: &ScopeState,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let env_id = self.resolve(variable_name.clone(), scope_state)?;
        let env = scope_state.get_scope(env_id).unwrap();
        let value = Arc::clone(env.variables.get(&variable_name).unwrap());
        Ok(value)
    }

    pub fn lookup_variable_safe(
        &self,
        variable_name: String,
        scope_state: &ScopeState,
    ) -> Option<Arc<Mutex<Box<dyn RuntimeValue>>>> {
        let env_id = self.resolve(variable_name.clone(), scope_state);
        if env_id.is_err() {
            return None;
        }
        let env = scope_state.get_scope(env_id.unwrap()).unwrap();
        env.variables.get(&variable_name).cloned()
    }

    // pub fn resolve_mut(
    //     &mut self,
    //     variable_name: String,
    // ) -> Result<EnvironmentId, InterpreterError> {
    //     if self.variables.contains_key(&variable_name) {
    //         return Ok(self.id);
    //     }

    //     if self.parent.is_none() {
    //         bail!(InterpreterError::UnresolvedVariable(variable_name))
    //     }

    //     let mut scope_state = SCOPE_STATE.lock().unwrap();
    //     let parent_scope = scope_state.get_scope_mut(self.parent.unwrap()).unwrap();
    //     let env_id = parent_scope.resolve_mut(variable_name)?;
    //     drop(scope_state);
    //     Ok(env_id)
    // }

    pub fn resolve(
        &self,
        variable_name: String,
        scope_state: &ScopeState,
    ) -> Result<EnvironmentId, InterpreterError> {
        if self.variables.contains_key(&variable_name) {
            return Ok(self.id);
        }

        if self.parent.is_none() {
            bail!(InterpreterError::UnresolvedVariable(variable_name))
        }

        // let scope_state = SCOPE_STATE.lock().unwrap();
        let parent_scope = scope_state.get_scope(self.parent.unwrap()).unwrap();
        let env_id = parent_scope.resolve(variable_name, scope_state)?;
        Ok(env_id)
    }
}
