use std::sync::{Arc, Mutex};

use pl_ast::{BinaryOperator, Node, UnaryOperator};

use crate::{stringify, values::RuntimeValue, EnvironmentId};

#[derive(Debug)]
pub enum InterpreterError {
    UnsupportedNode(Box<Node>),
    UnsupportedBinaryOperator(BinaryOperator),
    UnsupportedUnaryOperator(UnaryOperator),
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
    InvalidCondition(Box<dyn RuntimeValue>),
    InvalidValue(Box<dyn RuntimeValue>, String),
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
            InterpreterError::UnsupportedBinaryOperator(operator) => {
                write!(
                    f,
                    "This operator is not supported by interpreter yet: {:?}",
                    operator
                )
            }
            InterpreterError::UnsupportedUnaryOperator(operator) => {
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
                write!(
                    f,
                    "Invalid function callee: {:?}",
                    stringify(dyn_clone::clone_box(&**(value.lock().unwrap())))
                )
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
            InterpreterError::InvalidCondition(condition) => {
                write!(
                    f,
                    "Invalid condition: {:?}",
                    stringify(dyn_clone::clone_box(&**condition))
                )
            }
            Self::InvalidValue(value, expected) => {
                write!(
                    f,
                    "Invalid value: {}, expected {}",
                    stringify(dyn_clone::clone_box(&**value)),
                    expected
                )
            }
        }
    }
}
