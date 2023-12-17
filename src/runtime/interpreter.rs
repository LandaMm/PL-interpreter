use std::{
    collections::VecDeque,
    fmt::Debug,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use pl_ast::{AssignmentOperator, BinaryOperator, LogicalOperator, Node, UnaryOperator};

use crate::{
    cast_value,
    macros::bail,
    stringify,
    values::{DecimalValue, IntegerValue, NullValue, RuntimeValue, ValueType},
    ArrayValue, BoolValue, EnvironmentId, FunctionParameter, FunctionValue, NativeFnValue,
    ObjectValue, ScopeState, StringValue,
};

use super::error::InterpreterError;

lazy_static! {
    pub static ref SCOPE_STATE: Arc<Mutex<ScopeState>> = Arc::new(Mutex::new(ScopeState::new()));
}

pub struct Interpreter {
    stack: VecDeque<(Box<Node>, EnvironmentId)>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    pub fn run(&mut self, node: Box<Node>, env: EnvironmentId) -> Result<(), InterpreterError> {
        match *node {
            Node::Program(statements) | Node::BlockStatement(statements) => {
                for statement in statements {
                    self.stack.push_back((statement, env));
                }
            }
            Node::IfStatement(condition, body, alternate) => {
                self.eval_if_statement(condition, body, alternate, env)?;
            }
            Node::CallExpression(calle, args) => {
                self.eval_call_expression(calle, args, env)?;
            }
            // TODO: Add support for ForInStatement
            node => {
                self.resolve(Box::new(node), env)?;
            }
        }

        self.execute()?;

        Ok(())
    }

    pub fn execute(&mut self) -> Result<(), InterpreterError> {
        while let Some((current_node, current_env)) = self.stack.pop_front() {
            match *current_node {
                Node::CallExpression(calle, args) => {
                    self.eval_call_expression(calle, args, current_env)?;
                }
                Node::IfStatement(condition, body, alternate) => {
                    self.eval_if_statement(condition, body, alternate, current_env)?;
                }
                node => {
                    self.resolve(Box::new(node), current_env)?;
                }
            }
        }

        Ok(())
    }

    pub fn resolve(
        &mut self,
        node: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let value: Arc<Mutex<Box<dyn RuntimeValue>>> = match *node {
            Node::IntegerLiteral(value) => {
                Arc::new(Mutex::new(Box::new(IntegerValue::from(value as isize))))
            }
            Node::DecimalLiteral(value) => {
                Arc::new(Mutex::new(Box::new(DecimalValue::from(value))))
            }
            Node::StringLiteral(value) => Arc::new(Mutex::new(Box::new(StringValue::from(value)))),
            Node::ArrayExpression(items) => {
                let mut values: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>> = vec![];
                for item in items {
                    values.push(self.resolve(item, env)?);
                }
                Arc::new(Mutex::new(Box::new(ArrayValue::from(values))))
            }
            Node::BinaryExpression(..) => self.eval_binary_expression(node, env)?,
            Node::Identifier(identifier) => self.eval_identifier(identifier, env)?,
            Node::VariableDeclaration(variable_name, value, is_constant) => {
                self.eval_variable_declaration(variable_name, value, is_constant, env)?
            }
            Node::AssignmentExpression(left, operator, right) => {
                self.eval_assignment_expression(left, operator, right, env)?
            }
            Node::FunctionDeclaration(name, parameters, body) => {
                self.eval_function_declaration(name, parameters, body, env)?
            }
            Node::MemberExpression(object, property, computed) => {
                self.eval_member_expression(object, property, computed, env)?
            }
            Node::UnaryExpression(expression, operator) => {
                self.eval_unary_expression(expression, operator, env)?
            }
            Node::LogicalExpression(left, operator, right) => {
                self.eval_logical_expression(left, operator, right, env)?
            }
            Node::BlockStatement(statements) => {
                let mut stack: VecDeque<Box<Node>> = VecDeque::new();
                for statement in statements {
                    stack.push_back(statement);
                }
                let mut result: Arc<Mutex<Box<dyn RuntimeValue>>> =
                    Arc::new(Mutex::new(Box::new(NullValue::default())));
                while let Some(statement) = stack.pop_front() {
                    if let Node::ReturnStatement(value) = *statement {
                        result = self.resolve(value, env)?;
                        break;
                    } else {
                        self.resolve(statement, env)?;
                    }
                }
                result
            }
            Node::IfStatement(condition, body, alternate) => {
                self.eval_if_statement(condition, body, alternate, env)?
            }
            Node::CallExpression(calle, args) => self.eval_call_expression(calle, args, env)?,
            Node::ReturnStatement(..) => bail!(InterpreterError::UnexpectedNode(node)),
            node => bail!(InterpreterError::UnsupportedNode(Box::new(node))),
        };

        Ok(value)
    }

    fn eval_member_expression(
        &mut self,
        object: Box<Node>,
        property: Box<Node>,
        computed: bool,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let object = self.resolve(object, env)?;

        let obj = object.clone();
        let object_inner = obj.lock().unwrap();
        // TODO: add support for primitive types methods, e.g. for string
        // match object.kind() .... String => replace primitive with String class
        if object_inner.kind() != ValueType::Object {
            bail!(InterpreterError::UnsupportedValue(object))
        }

        let object = cast_value::<ObjectValue>(&object_inner).unwrap();

        let property: Arc<Mutex<Box<dyn RuntimeValue>>> = if computed {
            self.resolve(property, env)?
        } else {
            match *property {
                Node::Identifier(value) => Arc::new(Mutex::new(Box::new(StringValue::from(value)))),
                _ => bail!(InterpreterError::UnexpectedNode(property)),
            }
        };

        let prop = property.clone();
        let property_inner = prop.lock().unwrap();
        // TODO: add support for keys presented as numbers, e.g. { 10: "some value" }
        if property_inner.kind() != ValueType::String {
            bail!(InterpreterError::UnsupportedValue(property))
        }

        let key = cast_value::<StringValue>(&property_inner).unwrap().value();

        let map = object.map();

        if let Some(value) = map.get(&key) {
            return Ok(value.clone());
        }

        Ok(Arc::new(Mutex::new(Box::new(NullValue::default()))))
    }

    fn eval_unary_expression(
        &mut self,
        expression: Box<Node>,
        operator: UnaryOperator,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let target = self.resolve(expression.clone(), env)?;
        let result: Arc<Mutex<Box<dyn RuntimeValue>>> = match operator {
            UnaryOperator::Plus => target,
            UnaryOperator::Minus => {
                let target = target.lock().unwrap();
                match target.kind() {
                    ValueType::Decimal => {
                        let value = cast_value::<DecimalValue>(&target).unwrap();
                        Arc::new(Mutex::new(Box::new(DecimalValue::from(
                            value.value() * -1.0,
                        ))))
                    }
                    ValueType::Integer => {
                        let value = cast_value::<IntegerValue>(&target).unwrap();
                        Arc::new(Mutex::new(Box::new(IntegerValue::from(value.value() * -1))))
                    }
                    _ => bail!(InterpreterError::UnexpectedNode(expression)),
                }
            }
            UnaryOperator::Negation => {
                let target = target.lock().unwrap();
                match target.kind() {
                    ValueType::Boolean => {
                        let value = cast_value::<BoolValue>(&target).unwrap();
                        Arc::new(Mutex::new(Box::new(BoolValue::from(!value.value()))))
                    }
                    _ => bail!(InterpreterError::UnexpectedNode(expression)),
                }
            }
            _ => bail!(InterpreterError::UnsupportedUnaryOperator(operator)),
        };
        Ok(result)
    }

    fn eval_if_statement(
        &mut self,
        condition: Box<Node>,
        body: Box<Node>,
        alternate: Option<Box<Node>>,
        env_id: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let condition = self.resolve(condition, env_id)?;
        let condition = condition.lock().unwrap();
        if condition.kind() != ValueType::Boolean {
            bail!(InterpreterError::InvalidCondition(dyn_clone::clone_box(
                &**condition
            )))
        }

        let boolean = cast_value::<BoolValue>(&condition).unwrap();
        if boolean.value() {
            self.resolve(body, env_id)?;
        } else if let Some(alternate) = alternate {
            self.resolve(alternate, env_id)?;
        }
        return Ok(Arc::new(Mutex::new(Box::new(NullValue::default()))));
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
                            Some(self.resolve(right, env)?),
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
            let value = self.resolve(arg, env)?;
            args.push(value);
        }
        let fn_callee = self.resolve(callee, env)?;

        let fn_calle_c = fn_callee.clone();
        let fn_callee_box = fn_calle_c.lock().unwrap();
        let result: Arc<Mutex<Box<dyn RuntimeValue>>> = match fn_callee_box.kind() {
            ValueType::NativeFn => {
                let native_fn = match fn_callee_box.into_any().downcast::<NativeFnValue>() {
                    Ok(native_fn) => native_fn,
                    Err(_) => bail!(InterpreterError::UnsupportedValue(fn_callee.clone())),
                };
                let native_fn_c = dyn_clone::clone_box(&*native_fn);
                drop(native_fn);
                drop(fn_callee_box);
                drop(fn_callee);

                native_fn_c.callee().run(args.clone())
            }
            ValueType::Function => {
                let func = match fn_callee_box.into_any().downcast::<FunctionValue>() {
                    Ok(func) => func,
                    Err(_) => bail!(InterpreterError::UnsupportedValue(fn_callee.clone())),
                };
                let func_c = dyn_clone::clone_box(&*func);
                drop(func);
                drop(fn_callee_box);
                drop(fn_callee);
                let mut scope_state = SCOPE_STATE.lock().unwrap();
                let env_id = scope_state.create_environment(Some(func_c.declaration_env));
                let scope = scope_state.get_scope_mut(env_id).unwrap();
                for (index, parameter) in func_c.parameters.iter().enumerate() {
                    let value = args.get(index).unwrap().clone();
                    scope.declare_variable(parameter.name.clone(), value, true)?;
                }
                drop(scope_state);
                let value = self.resolve(func_c.body, env_id)?;
                value
                // Arc::new(Mutex::new(Box::new(NullValue::default())))
            }
            _ => bail!(InterpreterError::InvalidFunctionCallee(fn_callee.clone())),
        };
        Ok(result)
    }

    fn convert_value_to_node(&self, value: Box<dyn RuntimeValue>) -> Box<Node> {
        let node = match value.kind() {
            ValueType::Boolean => {
                let boolean = cast_value::<BoolValue>(&value).unwrap();
                Node::Identifier(if boolean.value() {
                    "true".to_string()
                } else {
                    "false".to_string()
                })
            }
            ValueType::Decimal => {
                let decimal = cast_value::<DecimalValue>(&value).unwrap();
                Node::DecimalLiteral(decimal.value())
            }
            ValueType::Integer => {
                let integer = cast_value::<IntegerValue>(&value).unwrap();
                if integer.value() < 0 {
                    Node::UnaryExpression(
                        Box::new(Node::IntegerLiteral(integer.value().abs() as usize)),
                        UnaryOperator::Minus,
                    )
                } else {
                    Node::IntegerLiteral(integer.value().abs() as usize)
                }
            }
            ValueType::Null => Node::Identifier("null".to_string()),
            ValueType::Function | ValueType::NativeFn => Node::Identifier(stringify(value)),
            ValueType::String => {
                let string = cast_value::<StringValue>(&value).unwrap();
                Node::StringLiteral(string.value())
            }
            ValueType::Array => {
                let array = cast_value::<ArrayValue>(&value).unwrap();
                Node::ArrayExpression(
                    array
                        .value()
                        .into_iter()
                        .map(|x| {
                            self.convert_value_to_node(dyn_clone::clone_box(&**x.lock().unwrap()))
                        })
                        .collect::<Vec<Box<Node>>>(),
                )
            }
            // TODO: throw InvalidOperand error instead
            _ => Node::Identifier("null".to_string()),
        };
        Box::new(node)
    }

    fn eval_logical_expression(
        &mut self,
        left: Box<Node>,
        operator: LogicalOperator,
        right: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        match operator {
            LogicalOperator::And => {
                let left = self.resolve(left, env)?;
                let right = self.resolve(right, env)?;
                let left_value = dyn_clone::clone_box(&**left.lock().unwrap());
                let right_value = dyn_clone::clone_box(&**right.lock().unwrap());
                if left_value.kind() != ValueType::Boolean {
                    bail!(InterpreterError::InvalidValue(
                        left_value,
                        "boolean".to_string()
                    ))
                }
                if left_value.kind() != ValueType::Boolean
                    || right_value.kind() != ValueType::Boolean
                {
                    bail!(InterpreterError::InvalidValue(
                        right_value,
                        "boolean".to_string()
                    ))
                }
                let left_bool = cast_value::<BoolValue>(&left_value).unwrap();
                let right_bool = cast_value::<BoolValue>(&right_value).unwrap();
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left_bool.value() && right_bool.value(),
                )))));
            }
            LogicalOperator::Or => {
                let left = self.resolve(left, env)?;
                let right = self.resolve(right, env)?;
                let left_value = dyn_clone::clone_box(&**left.lock().unwrap());
                let right_value = dyn_clone::clone_box(&**right.lock().unwrap());
                if left_value.kind() != ValueType::Boolean {
                    bail!(InterpreterError::InvalidValue(
                        left_value,
                        "boolean".to_string()
                    ))
                }
                if left_value.kind() != ValueType::Boolean
                    || right_value.kind() != ValueType::Boolean
                {
                    bail!(InterpreterError::InvalidValue(
                        right_value,
                        "boolean".to_string()
                    ))
                }
                let left_bool = cast_value::<BoolValue>(&left_value).unwrap();
                let right_bool = cast_value::<BoolValue>(&right_value).unwrap();
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left_bool.value() || right_bool.value(),
                )))));
            }
        }
    }

    fn assign_variable(
        &mut self,
        name: String,
        value: Arc<Mutex<Box<dyn RuntimeValue>>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut scope_state = SCOPE_STATE.lock().unwrap();
        Ok(scope_state.assign_variable(name, value, env)?)
    }

    fn eval_assignment_expression(
        &mut self,
        left: Box<Node>,
        operator: AssignmentOperator,
        right: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        if let Node::Identifier(variable_name) = *left {
            let value = match operator {
                AssignmentOperator::Equals => {
                    let right = self.resolve(right, env)?;
                    self.assign_variable(variable_name.clone(), right, env)?
                }
                AssignmentOperator::Addition => {
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env).unwrap();
                    let previous_value =
                        scope.lookup_variable(variable_name.clone(), &scope_state)?;
                    let left = self.convert_value_to_node(dyn_clone::clone_box(
                        &**previous_value.lock().unwrap(),
                    ));
                    let binary = Node::BinaryExpression(left, BinaryOperator::Plus, right);
                    let value = self.resolve(Box::new(binary), env)?;
                    drop(scope_state);
                    self.assign_variable(variable_name.clone(), value, env)?
                }
                AssignmentOperator::Division => {
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env).unwrap();
                    let previous_value =
                        scope.lookup_variable(variable_name.clone(), &scope_state)?;
                    let left = self.convert_value_to_node(dyn_clone::clone_box(
                        &**previous_value.lock().unwrap(),
                    ));
                    let binary = Node::BinaryExpression(left, BinaryOperator::Divide, right);
                    let value = self.resolve(Box::new(binary), env)?;
                    drop(scope_state);
                    self.assign_variable(variable_name.clone(), value, env)?
                }
                AssignmentOperator::Modulation => {
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env).unwrap();
                    let previous_value =
                        scope.lookup_variable(variable_name.clone(), &scope_state)?;
                    let left = self.convert_value_to_node(dyn_clone::clone_box(
                        &**previous_value.lock().unwrap(),
                    ));
                    let binary = Node::BinaryExpression(left, BinaryOperator::Modulo, right);
                    let value = self.resolve(Box::new(binary), env)?;
                    drop(scope_state);
                    self.assign_variable(variable_name.clone(), value, env)?
                }
                AssignmentOperator::Subtraction => {
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env).unwrap();
                    let previous_value =
                        scope.lookup_variable(variable_name.clone(), &scope_state)?;
                    let left = self.convert_value_to_node(dyn_clone::clone_box(
                        &**previous_value.lock().unwrap(),
                    ));
                    let binary = Node::BinaryExpression(left, BinaryOperator::Minus, right);
                    let value = self.resolve(Box::new(binary), env)?;
                    drop(scope_state);
                    self.assign_variable(variable_name.clone(), value, env)?
                }
                AssignmentOperator::Multiplication => {
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env).unwrap();
                    let previous_value =
                        scope.lookup_variable(variable_name.clone(), &scope_state)?;
                    let left = self.convert_value_to_node(dyn_clone::clone_box(
                        &**previous_value.lock().unwrap(),
                    ));
                    let binary = Node::BinaryExpression(left, BinaryOperator::Multiply, right);
                    let value = self.resolve(Box::new(binary), env)?;
                    drop(scope_state);
                    self.assign_variable(variable_name.clone(), value, env)?
                }
            };
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
            Some(value) => self.resolve(value, env)?,
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
            BinaryOperator::GreaterThan => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left > right,
                )))))
            }
            BinaryOperator::LessThan => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left < right,
                )))))
            }
            BinaryOperator::NotEquals => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left != right,
                )))))
            }
            BinaryOperator::IsEquals => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left == right,
                )))))
            }
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
        let left: isize = left.into();
        let right: isize = right.into();

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
            BinaryOperator::GreaterThan => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left > right,
                )))))
            }
            BinaryOperator::LessThan => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left < right,
                )))))
            }
            BinaryOperator::NotEquals => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left != right,
                )))))
            }
            BinaryOperator::IsEquals => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left == right,
                )))))
            }
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
            let left = self.resolve(left, env)?;
            let right = self.resolve(right, env)?;

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
