use pl_ast::{BinaryOperator, Node};

use crate::values::RuntimeValue;

#[derive(Debug)]
pub enum InterpreterError {
    UnsupportedNode(Box<Node>),
    UnsupportedOperator(BinaryOperator),
    UnsupportedValue(Box<dyn RuntimeValue>),
    UnexpectedNode(Box<Node>),
    ValueCastError(Box<dyn RuntimeValue>, String),
    VariableDeclarationExist(String),
    UnresolvedVariable(String),
}

impl std::fmt::Display for InterpreterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterpreterError::UnsupportedNode(node) => {
                write!(
                    f,
                    "This node is not supported by interpreter yet: {:?}",
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
                write!(f, "Cannot resolve {variable_name} as it doesn't exist")
            }
        }
    }
}
