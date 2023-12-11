use pl_ast::{AssignmentOperator, BinaryOperator, Node};

use crate::{
    macros::bail,
    values::{DecimalValue, IntegerValue, NullValue, RuntimeValue, ValueType},
    Environment, NativeFnValue,
};

use super::error::InterpreterError;

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &self,
        node: Box<Node>,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        match *node {
            Node::IntegerLiteral(value) => return Ok(Box::new(IntegerValue::from(value as isize))),
            Node::DecimalLiteral(value) => return Ok(Box::new(DecimalValue::from(value))),
            Node::NullLiteral() => return Ok(Box::new(NullValue::default())),
            Node::Program(program) => return Ok(self.eval_program(program, env)?),
            Node::BinaryExpression(..) => return Ok(self.eval_binary_expression(node, env)?),
            Node::Identifier(identifier) => return Ok(self.eval_identifier(identifier, env)?),
            Node::VariableDeclaration(variable_name, value, is_constant) => {
                return Ok(self.eval_variable_declaration(
                    variable_name,
                    value,
                    is_constant,
                    env,
                )?)
            }
            Node::AssignmentExpression(left, operator, right) => {
                return Ok(self.eval_assignment_expression(left, operator, right, env)?)
            }
            Node::CallExpression(callee, arguments) => {
                return Ok(self.eval_call_expression(callee, arguments, env)?)
            }
            node => bail!(InterpreterError::UnsupportedNode(Box::new(node))),
        }
    }

    fn eval_call_expression(
        &self,
        callee: Box<Node>,
        arguments: Vec<Box<Node>>,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        let mut args: Vec<Box<dyn RuntimeValue>> = vec![];
        for arg in arguments {
            let value = self.evaluate(arg, env)?;
            args.push(value);
        }

        let fn_callee = self.evaluate(callee, env)?;

        if fn_callee.kind() != ValueType::NativeFn {
            bail!(InterpreterError::InvalidFunctionCallee(fn_callee))
        }

        let fn_callee_clone = dyn_clone::clone_box(&*fn_callee);

        let native_fn = match fn_callee.into_any().downcast::<NativeFnValue>() {
            Ok(native_fn) => native_fn,
            Err(_) => bail!(InterpreterError::UnsupportedValue(fn_callee_clone)),
        };

        let function_call = native_fn.call.borrow();
        let result = function_call(args, env);

        Ok(result)
    }

    fn eval_assignment_expression(
        &self,
        left: Box<Node>,
        // TODO: add support for operator
        operator: AssignmentOperator,
        right: Box<Node>,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        if let Node::Identifier(variable_name) = *left {
            let right = self.evaluate(right, env)?;
            let value = env.assign_variable(variable_name, right)?;
            let new_value = dyn_clone::clone_box(&**value);
            return Ok(new_value);
        }
        bail!(InterpreterError::InvalidAssignFactor(left))
    }

    fn eval_variable_declaration(
        &self,
        variable_name: String,
        value: Option<Box<Node>>,
        is_constant: bool,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        let value = match value {
            Some(value) => self.evaluate(value, env)?,
            None => Box::new(NullValue::default()),
        };

        env.declare_variable(variable_name.clone(), value, is_constant)?;

        Ok(dyn_clone::clone_box(&**env.lookup_variable(variable_name)?))
    }

    fn eval_identifier(
        &self,
        identifier: String,
        env: &Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        let val = env.lookup_variable(identifier)?;
        Ok(dyn_clone::clone_box(&**val))
    }

    fn eval_statements(
        &self,
        statements: Vec<Box<Node>>,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        let mut last_evaluated: Box<dyn RuntimeValue> = Box::new(NullValue::default());

        for statement in statements {
            last_evaluated = self.evaluate(statement, env)?;
        }

        Ok(last_evaluated)
    }

    fn eval_program(
        &self,
        statements: Vec<Box<Node>>,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        return Ok(self.eval_statements(statements, env)?);
    }

    fn get_integer_value(
        &self,
        value: Box<dyn RuntimeValue>,
    ) -> Result<Box<IntegerValue>, InterpreterError> {
        let value_clone = dyn_clone::clone_box(&*value);
        if value.kind() == ValueType::Integer {
            let any = value.into_any();
            match any.downcast::<IntegerValue>() {
                Ok(value) => return Ok(value),
                Err(_) => bail!(InterpreterError::ValueCastError(
                    value_clone,
                    "integer".to_string()
                )),
            }
        }
        bail!(InterpreterError::ValueCastError(
            value,
            "integer".to_string()
        ))
    }

    fn get_decimal_value(
        &self,
        value: Box<dyn RuntimeValue>,
    ) -> Result<Box<DecimalValue>, InterpreterError> {
        let value_clone = dyn_clone::clone_box(&*value);
        if value.kind() == ValueType::Decimal {
            let any = value.into_any();
            match any.downcast::<DecimalValue>() {
                Ok(value) => return Ok(value),
                Err(_) => bail!(InterpreterError::ValueCastError(
                    value_clone,
                    "decimal".to_string()
                )),
            }
        }
        if value.kind() == ValueType::Integer {
            let any = value.into_any();
            match any.downcast::<IntegerValue>() {
                Ok(value) => return Ok(Box::new(DecimalValue::from(value.value() as f64))),
                Err(_) => bail!(InterpreterError::ValueCastError(
                    value_clone,
                    "decimal".to_string()
                )),
            }
        }
        bail!(InterpreterError::ValueCastError(
            value,
            "decimal".to_string()
        ))
    }

    fn eval_decimals<T>(
        &self,
        left: T,
        right: T,
        operator: BinaryOperator,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError>
    where
        T: Into<f64>,
    {
        let left: f64 = left.into();
        let right: f64 = right.into();

        match operator {
            BinaryOperator::Plus => return Ok(Box::new(DecimalValue::from(left + right))),
            BinaryOperator::Minus => return Ok(Box::new(DecimalValue::from(left - right))),
            BinaryOperator::Divide => return Ok(Box::new(DecimalValue::from(left / right))),
            BinaryOperator::Multiply => return Ok(Box::new(DecimalValue::from(left * right))),
            BinaryOperator::Modulo => return Ok(Box::new(DecimalValue::from(left % right))),
            _ => bail!(InterpreterError::UnsupportedOperator(operator)),
        }
    }

    fn eval_integers<T>(
        &self,
        left: T,
        right: T,
        operator: BinaryOperator,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError>
    where
        T: Into<isize>,
    {
        let left: isize = left.into();
        let right: isize = right.into();

        match operator {
            BinaryOperator::Plus => return Ok(Box::new(IntegerValue::from(left + right))),
            BinaryOperator::Minus => return Ok(Box::new(IntegerValue::from(left - right))),
            BinaryOperator::Divide => return Ok(Box::new(IntegerValue::from(left / right))),
            BinaryOperator::Multiply => return Ok(Box::new(IntegerValue::from(left * right))),
            BinaryOperator::Modulo => return Ok(Box::new(IntegerValue::from(left % right))),
            _ => bail!(InterpreterError::UnsupportedOperator(operator)),
        }
    }

    fn eval_numeric_binary_expression(
        &self,
        left: Box<dyn RuntimeValue>,
        right: Box<dyn RuntimeValue>,
        operator: BinaryOperator,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        if left.kind() == ValueType::Decimal || right.kind() == ValueType::Decimal {
            // we are trying to add 1-2 decimals -> will get decimal
            let left = self.get_decimal_value(left)?;
            let right = self.get_decimal_value(right)?;

            return Ok(self.eval_decimals(left.value(), right.value(), operator)?);
        }
        if left.kind() == ValueType::Integer && right.kind() == ValueType::Integer {
            let left = self.get_integer_value(left)?;
            let right = self.get_integer_value(right)?;

            return Ok(self.eval_integers(left.value(), right.value(), operator)?);
        }

        if left.kind() != ValueType::Integer {
            bail!(InterpreterError::UnsupportedValue(left))
        } else {
            bail!(InterpreterError::UnsupportedValue(right))
        }
    }

    fn eval_binary_expression(
        &self,
        node: Box<Node>,
        env: &mut Environment,
    ) -> Result<Box<dyn RuntimeValue>, InterpreterError> {
        if let Node::BinaryExpression(left, operator, right) = *node {
            let left = self.evaluate(left, env)?;
            let right = self.evaluate(right, env)?;

            if (left.kind() == ValueType::Integer || left.kind() == ValueType::Decimal)
                && (right.kind() == ValueType::Integer || right.kind() == ValueType::Decimal)
            {
                return Ok(self.eval_numeric_binary_expression(left, right, operator)?);
            }

            return Ok(Box::new(NullValue::default()));
        } else {
            bail!(InterpreterError::UnexpectedNode(node))
        }
    }
}
