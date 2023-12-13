use std::sync::{Arc, Mutex};

use pl_ast::{BinaryOperator, Node};

use crate::{values::RuntimeValue, EnvironmentId};

#[derive(Debug)]
pub enum InterpreterError {
    UnsupportedNode(Box<Node>),
    UnsupportedOperator(BinaryOperator),
    UnsupportedValue(Arc<Mutex<Box<dyn RuntimeValue>>>),
    UnexpectedNode(Box<Node>),
    ValueCastError(Box<dyn RuntimeValue>, String),
    VariableDeclarationExist(String),
    UnresolvedVariable(String),
    ReassignConstant(String),
    InvalidAssignFactor(Box<Node>),
    InvalidFunctionCallee(Arc<Mutex<Box<dyn RuntimeValue>>>),
    InvalidFunctionParameter(Box<Node>),
    InvalidFunctionEnvironment(EnvironmentId),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::UnsupportedNode(node) => {
                write!(
                    f,
                    "This node is not supported by interpreter yet: {:#?}",
                    node
                )
            }
            InterpreterError::UnexpectedNode(node) => {
                write!(f, "Unexpected node: {:?}", node)
            }
            InterpreterError::ValueCastError(node, cast) => {
                write!(f, "Failed to cast node to {}: {:?}", cast, node)
            }
            InterpreterError::UnsupportedOperator(operator) => {
                write!(
                    f,
                    "This operator is not supported by interpreter yet: {:?}",
                    operator
                )
            }
            InterpreterError::UnsupportedValue(value) => {
                write!(f, "Unsupported value: {:?}", value)
            }
            InterpreterError::VariableDeclarationExist(variable_name) => {
                write!(
                    f,
                    "Cannot re-declare variable \"{variable_name}\" as it's already defined"
                )
            }
            InterpreterError::UnresolvedVariable(variable_name) => {
                write!(f, "Cannot resolve \"{variable_name}\" as it doesn't exist")
            }
            InterpreterError::ReassignConstant(variable_name) => {
                write!(f, "Cannot reassign \"{variable_name}\" as it's a constant")
            }
            InterpreterError::InvalidAssignFactor(factor) => {
                write!(
                    f,
                    "Invalid left-hand side for assignment expression: {:?}",
                    factor
                )
            }
            InterpreterError::InvalidFunctionCallee(value) => {
                write!(f, "Invalid function callee: {value:?}")
            }
            InterpreterError::InvalidFunctionParameter(parameter) => {
                write!(f, "Invalid parameter provided in function: {parameter:?}")
            }
            InterpreterError::InvalidFunctionEnvironment(env_id) => {
                write!(
                    f,
                    "Environment (scope) for function is not found with given id: {:?}",
                    env_id
                )
            }
        }
    }
}
