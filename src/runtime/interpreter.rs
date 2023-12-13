use std::{
    fmt::Debug,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use pl_ast::{AssignmentOperator, BinaryOperator, Node};

use crate::{
    macros::bail,
    values::{DecimalValue, IntegerValue, NullValue, RuntimeValue, ValueType},
    EnvironmentId, FunctionParameter, FunctionValue, NativeFnValue, ScopeState,
};

use super::error::InterpreterError;

lazy_static! {
    pub static ref SCOPE_STATE: Arc<Mutex<ScopeState>> = Arc::new(Mutex::new(ScopeState::new()));
}

pub struct Interpreter;

impl Interpreter {
    pub fn new() -> Self {
        Self
    }

    pub fn evaluate(
        &mut self,
        node: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        match *node {
            Node::IntegerLiteral(value) => {
                return Ok(Arc::new(Mutex::new(Box::new(IntegerValue::from(
                    value as isize,
                )))))
            }
            Node::DecimalLiteral(value) => {
                return Ok(Arc::new(Mutex::new(Box::new(DecimalValue::from(value)))))
            }
            Node::NullLiteral() => return Ok(Arc::new(Mutex::new(Box::new(NullValue::default())))),
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
            Node::FunctionDeclaration(name, parameters, body) => {
                return Ok(self.eval_function_declaration(name, parameters, body, env)?)
            }
            Node::BlockStatement(statements) => {
                return Ok(self.eval_block_statement(statements, env)?)
            }
            node => bail!(InterpreterError::UnsupportedNode(Box::new(node))),
        }
    }

    fn eval_block_statement(
        &mut self,
        statements: Vec<Box<Node>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut last_evaluated: Arc<Mutex<Box<dyn RuntimeValue>>> =
            Arc::new(Mutex::new(Box::new(NullValue::default())));
        for stmt in statements {
            last_evaluated = self.evaluate(stmt, env)?;
        }
        Ok(last_evaluated)
    }

    fn eval_function_declaration(
        &mut self,
        name: String,
        params: Vec<Box<Node>>,
        body: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut parameters: Vec<FunctionParameter> = vec![];
        for parameter in params {
            let parameter_clone = dyn_clone::clone_box(&*parameter);
            match *parameter {
                Node::Identifier(value) => {
                    parameters.push(FunctionParameter::new(value, None));
                }
                Node::AssignmentExpression(left, operator, right) => {
                    if operator != AssignmentOperator::Equals {
                        bail!(InterpreterError::InvalidFunctionParameter(parameter_clone))
                    }
                    let left_clone = dyn_clone::clone_box(&*left);
                    if let Node::Identifier(value) = *left {
                        parameters.push(FunctionParameter::new(
                            value,
                            Some(self.evaluate(right, env)?),
                        ));
                    }
                    bail!(InterpreterError::InvalidAssignFactor(left_clone))
                }
                _ => bail!(InterpreterError::InvalidFunctionParameter(parameter)),
            }
        }
        let function = FunctionValue::new(name.clone(), parameters, env, body);

        let scope_c = SCOPE_STATE.clone();
        let mut scope_state = scope_c.lock().unwrap();
        let scope = match scope_state.get_scope_mut(env) {
            Some(scope) => scope,
            None => bail!(InterpreterError::InvalidFunctionEnvironment(env)),
        };

        let value =
            scope.declare_variable(name.clone(), Arc::new(Mutex::new(Box::new(function))), true)?;

        Ok(value)
    }

    fn eval_call_expression(
        &mut self,
        callee: Box<Node>,
        arguments: Vec<Box<Node>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = vec![];
        for arg in arguments {
            let value = self.evaluate(arg, env)?;
            args.push(value);
        }
        let fn_callee = self.evaluate(callee, env)?;

        let fn_callee_box = fn_callee.lock().unwrap();
        let result = match fn_callee_box.kind() {
            ValueType::NativeFn => {
                let native_fn = match fn_callee_box.into_any().downcast::<NativeFnValue>() {
                    Ok(native_fn) => native_fn,
                    Err(_) => bail!(InterpreterError::UnsupportedValue(fn_callee.clone())),
                };

                native_fn.callee().run(args)
            }
            ValueType::Function => {
                let func = match fn_callee_box.into_any().downcast::<FunctionValue>() {
                    Ok(func) => func,
                    Err(_) => bail!(InterpreterError::UnsupportedValue(fn_callee.clone())),
                };
                let mut scope_state = SCOPE_STATE.lock().unwrap();
                let env_id = scope_state.create_environment(Some(func.declaration_env));
                let scope = scope_state.get_scope_mut(env_id).unwrap();
                for (index, parameter) in func.parameters.iter().enumerate() {
                    let value = args.get(index).unwrap().clone();
                    scope.declare_variable(parameter.name.clone(), value, true)?;
                }
                drop(scope_state);
                self.evaluate(func.body, env_id)?
            }
            _ => bail!(InterpreterError::InvalidFunctionCallee(fn_callee.clone())),
        };
        Ok(result)
    }

    fn eval_assignment_expression(
        &mut self,
        left: Box<Node>,
        // TODO: add support for operator
        operator: AssignmentOperator,
        right: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        if let Node::Identifier(variable_name) = *left {
            // TODO: somehow drop SCOPE_STATE
            let scope_c = SCOPE_STATE.clone();
            let mut scope_state = scope_c.lock().unwrap();
            let right = self.evaluate(right, env)?;
            let value = scope_state.assign_variable(variable_name, right, env)?;
            drop(scope_state);
            return Ok(value);
        }
        bail!(InterpreterError::InvalidAssignFactor(left))
    }

    fn eval_variable_declaration(
        &mut self,
        variable_name: String,
        value: Option<Box<Node>>,
        is_constant: bool,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let value: Arc<Mutex<Box<dyn RuntimeValue>>> = match value {
            Some(value) => self.evaluate(value, env)?,
            None => Arc::new(Mutex::new(Box::new(NullValue::default()))),
        };

        let scope_c = SCOPE_STATE.clone();
        let mut scope_state = scope_c.lock().unwrap();
        let scope = match scope_state.get_scope_mut(env) {
            Some(scope) => scope,
            None => bail!(InterpreterError::InvalidFunctionEnvironment(env)),
        };

        let value = scope.declare_variable(variable_name.clone(), value, is_constant)?;
        drop(scope_state);

        Ok(value)
    }

    fn eval_identifier(
        &mut self,
        identifier: String,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let scope_state = SCOPE_STATE.lock().unwrap();
        let scope = match scope_state.get_scope(env) {
            Some(scope) => scope,
            None => bail!(InterpreterError::InvalidFunctionEnvironment(env)),
        };
        let val = scope.lookup_variable(identifier, &scope_state)?;
        drop(scope_state);
        Ok(val)
    }

    fn eval_statements(
        &mut self,
        statements: Vec<Box<Node>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut last_evaluated: Arc<Mutex<Box<dyn RuntimeValue>>> =
            Arc::new(Mutex::new(Box::new(NullValue::default())));

        for statement in statements {
            last_evaluated = self.evaluate(statement, env)?;
        }

        Ok(last_evaluated)
    }

    fn eval_program(
        &mut self,
        statements: Vec<Box<Node>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        return Ok(self.eval_statements(statements, env)?);
    }

    fn get_integer_value(
        &mut self,
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
        &mut self,
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
        &mut self,
        left: T,
        right: T,
        operator: BinaryOperator,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError>
    where
        T: Into<f64>,
    {
        let left: f64 = left.into();
        let right: f64 = right.into();

        match operator {
            BinaryOperator::Plus => {
                return Ok(Arc::new(Mutex::new(Box::new(DecimalValue::from(
                    left + right,
                )))))
            }
            BinaryOperator::Minus => {
                return Ok(Arc::new(Mutex::new(Box::new(DecimalValue::from(
                    left - right,
                )))))
            }
            BinaryOperator::Divide => {
                return Ok(Arc::new(Mutex::new(Box::new(DecimalValue::from(
                    left / right,
                )))))
            }
            BinaryOperator::Multiply => {
                return Ok(Arc::new(Mutex::new(Box::new(DecimalValue::from(
                    left * right,
                )))))
            }
            BinaryOperator::Modulo => {
                return Ok(Arc::new(Mutex::new(Box::new(DecimalValue::from(
                    left % right,
                )))))
            }
            _ => bail!(InterpreterError::UnsupportedOperator(operator)),
        }
    }

    fn eval_integers<T>(
        &mut self,
        left: T,
        right: T,
        operator: BinaryOperator,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError>
    where
        T: Into<isize> + Debug,
    {
        println!("eval-integers-before: {:?} {:?}", left, right);
        println!("operator: {:?}", operator);

        let left: isize = left.into();
        let right: isize = right.into();

        println!("eval-integers: {} {}", left, right);

        match operator {
            BinaryOperator::Plus => {
                return Ok(Arc::new(Mutex::new(Box::new(IntegerValue::from(
                    left + right,
                )))))
            }
            BinaryOperator::Minus => {
                return Ok(Arc::new(Mutex::new(Box::new(IntegerValue::from(
                    left - right,
                )))))
            }
            BinaryOperator::Divide => {
                return Ok(Arc::new(Mutex::new(Box::new(IntegerValue::from(
                    left / right,
                )))))
            }
            BinaryOperator::Multiply => {
                return Ok(Arc::new(Mutex::new(Box::new(IntegerValue::from(
                    left * right,
                )))))
            }
            BinaryOperator::Modulo => {
                return Ok(Arc::new(Mutex::new(Box::new(IntegerValue::from(
                    left % right,
                )))))
            }
            _ => bail!(InterpreterError::UnsupportedOperator(operator)),
        }
    }

    fn eval_numeric_binary_expression(
        &mut self,
        left: Arc<Mutex<Box<dyn RuntimeValue>>>,
        right: Arc<Mutex<Box<dyn RuntimeValue>>>,
        operator: BinaryOperator,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let left = left.lock().unwrap();
        let right = right.lock().unwrap();
        if left.kind() == ValueType::Decimal || right.kind() == ValueType::Decimal {
            // we are trying to add 1-2 decimals -> will get decimal
            let left = self.get_decimal_value(dyn_clone::clone_box(&**left))?;
            let right = self.get_decimal_value(dyn_clone::clone_box(&**right))?;

            return Ok(self.eval_decimals(left.value(), right.value(), operator)?);
        }
        if left.kind() == ValueType::Integer && right.kind() == ValueType::Integer {
            println!("eval-numeric: {:?}, {:?}", left, right);
            let left = self.get_integer_value(dyn_clone::clone_box(&**left))?;
            let right = self.get_integer_value(dyn_clone::clone_box(&**right))?;

            return Ok(self.eval_integers(left.value(), right.value(), operator)?);
        }

        if left.kind() != ValueType::Integer {
            bail!(InterpreterError::UnsupportedValue(Arc::new(Mutex::new(
                dyn_clone::clone_box(&**left)
            ))))
        } else {
            bail!(InterpreterError::UnsupportedValue(Arc::new(Mutex::new(
                dyn_clone::clone_box(&**right)
            ))))
        }
    }

    fn eval_binary_expression(
        &mut self,
        node: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        if let Node::BinaryExpression(left, operator, right) = *node {
            let left = self.evaluate(left, env)?;
            let right = self.evaluate(right, env)?;

            let left_kind = left.lock().unwrap().kind();
            let right_kind = right.lock().unwrap().kind();

            if (left_kind == ValueType::Integer || left_kind == ValueType::Decimal)
                && (right_kind == ValueType::Integer || right_kind == ValueType::Decimal)
            {
                return Ok(self.eval_numeric_binary_expression(left, right, operator)?);
            }

            return Ok(Arc::new(Mutex::new(Box::new(NullValue::default()))));
        } else {
            bail!(InterpreterError::UnexpectedNode(node))
        }
    }
}
