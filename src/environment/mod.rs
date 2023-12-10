use std::collections::{HashMap, HashSet};

use crate::{macros::bail, InterpreterError, RuntimeValue};

#[derive(Debug)]
pub struct Environment {
    parent: Option<Box<Environment>>,
    variables: HashMap<String, Box<dyn RuntimeValue>>,
    constants: HashSet<String>,
}

impl Environment {
    pub fn new(parent_env: Option<Box<Environment>>) -> Self {
        Self {
            parent: parent_env,
            variables: HashMap::new(),
            constants: HashSet::new(),
        }
    }

    pub fn declare_variable(
        &mut self,
        variable_name: String,
        value: Box<dyn RuntimeValue>,
        is_constant: bool,
    ) -> Result<&Box<dyn RuntimeValue>, InterpreterError> {
        if self.variables.contains_key(&variable_name) {
            bail!(InterpreterError::VariableDeclarationExist(variable_name))
        }

        self.variables.insert(variable_name.clone(), value);

        if is_constant {
            self.constants.insert(variable_name.clone());
        }

        Ok(self.variables.get(&variable_name).unwrap())
    }

    pub fn assign_variable(
        &mut self,
        variable_name: String,
        value: Box<dyn RuntimeValue>,
    ) -> Result<&Box<dyn RuntimeValue>, InterpreterError> {
        let env = self.resolve_mut(variable_name.clone())?;

        if env.constants.contains(&variable_name) {
            bail!(InterpreterError::ReassignConstant(variable_name.clone()))
        }

        env.variables
            .entry(variable_name.clone())
            .and_modify(|val| *val = value);
        Ok(env.variables.get(&variable_name).unwrap())
    }

    pub fn lookup_variable(
        &self,
        variable_name: String,
    ) -> Result<&Box<dyn RuntimeValue>, InterpreterError> {
        let env = self.resolve(variable_name.clone())?;
        Ok(env.variables.get(&variable_name).unwrap())
    }

    fn resolve_mut(
        &mut self,
        variable_name: String,
    ) -> Result<Box<&mut Environment>, InterpreterError> {
        if self.variables.contains_key(&variable_name) {
            return Ok(Box::new(self));
        }

        if self.parent.is_none() {
            bail!(InterpreterError::UnresolvedVariable(variable_name))
        }

        self.parent.as_mut().unwrap().resolve_mut(variable_name)
    }

    fn resolve(&self, variable_name: String) -> Result<Box<&Environment>, InterpreterError> {
        if self.variables.contains_key(&variable_name) {
            return Ok(Box::new(self));
        }

        if self.parent.is_none() {
            bail!(InterpreterError::UnresolvedVariable(variable_name))
        }

        self.parent.as_ref().unwrap().resolve(variable_name)
    }
}
