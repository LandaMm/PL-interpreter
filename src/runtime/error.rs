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
    UnexpectedValue(Box<dyn RuntimeValue>),
    ValueCastError(Box<dyn RuntimeValue>, String),
    VariableDeclarationExist(String),
    UnresolvedVariable(String),
    UnresolvedProperty(String),
    ReassignConstant(String),
    InvalidAssignFactor(Box<Node>),
    InvalidFunctionCallee(Arc<Mutex<Box<dyn RuntimeValue>>>),
    InvalidFunctionParameter(Box<Node>),
    InvalidCondition(Box<dyn RuntimeValue>),
    InvalidValue(Box<dyn RuntimeValue>, String),
    InvalidDefaultParameter(String),
    InvalidParameterCount(usize, usize),
    UnresolvedEnvironment(EnvironmentId),
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
                write!(
                    f,
                    "Unsupported value: {}",
                    stringify(dyn_clone::clone_box(
                        &**value
                            .lock()
                            .expect("error: failed to get unsupported value")
                    ))
                )
            }
            InterpreterError::UnexpectedValue(value) => {
                write!(
                    f,
                    "Unexpected value: {}",
                    stringify(dyn_clone::clone_box(&**value))
                )
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
                    stringify(dyn_clone::clone_box(
                        &**(value
                            .lock()
                            .expect("error: failed to get function callee value"))
                    ))
                )
            }
            InterpreterError::InvalidFunctionParameter(parameter) => {
                write!(f, "Invalid parameter provided in function: {parameter:?}")
            }
            InterpreterError::InvalidCondition(condition) => {
                write!(
                    f,
                    "Invalid condition: {:?}",
                    stringify(dyn_clone::clone_box(&**condition))
                )
            }
            InterpreterError::InvalidValue(value, expected) => {
                write!(
                    f,
                    "Invalid value: {}, expected {}",
                    stringify(dyn_clone::clone_box(&**value)),
                    expected
                )
            }
            InterpreterError::InvalidDefaultParameter(name) => {
                write!(
                    f,
                    "Invalid default value provided for non-last argument: {}",
                    name
                )
            }
            InterpreterError::InvalidParameterCount(expected, got) => {
                write!(
                    f,
                    "Invalid calle parameters count. Expected {}, but got {}",
                    expected, got
                )
            }
            InterpreterError::UnresolvedProperty(property) => {
                write!(f, "Unresolved property: {}", property)
            }
            Self::UnresolvedEnvironment(env) => {
                write!(f, "Unresolved environment: {}", env)
            }
        }
    }
}
