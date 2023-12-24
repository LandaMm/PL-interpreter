use std::{
    collections::{HashMap, VecDeque},
    fmt::Debug,
    sync::{Arc, Mutex},
};

use lazy_static::lazy_static;
use pl_ast::{AssignmentOperator, BinaryOperator, LogicalOperator, Node, UnaryOperator};

use crate::{
    cast_value, get_number_object, get_string_object,
    macros::bail,
    stringify,
    values::{DecimalValue, IntegerValue, NullValue, RuntimeValue, ValueType},
    ArrayValue, BoolValue, ClassMethod, ClassMethodParameter, ClassProperty, ClassValue,
    EnvironmentId, FunctionParameter, FunctionValue, Key, NativeFnValue, ObjectValue, ScopeState,
    StringValue, Value,
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
            Node::WhileStatement(condition, consequent) => {
                self.eval_while_statement(condition, consequent, env)?;
            }
            Node::CallExpression(calle, args) => {
                self.eval_call_expression(calle, args, env)?;
            }
            Node::ClassDeclaration(id, super_class, body) => {
                self.eval_class_declaration(id, super_class, body, env)?;
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
                Node::WhileStatement(condition, consequent) => {
                    self.eval_while_statement(condition, consequent, current_env)?;
                }
                Node::ClassDeclaration(id, super_class, body) => {
                    self.eval_class_declaration(id, super_class, body, current_env)?;
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
            Node::WhileStatement(condition, consequent) => {
                self.eval_while_statement(condition, consequent, env)?
            }
            Node::CallExpression(calle, args) => self.eval_call_expression(calle, args, env)?,
            Node::ReturnStatement(..) => bail!(InterpreterError::UnexpectedNode(node)),
            node => bail!(InterpreterError::UnsupportedNode(Box::new(node))),
        };

        let v = value.lock().unwrap();
        let res = Arc::new(Mutex::new(dyn_clone::clone_box(&**v)));

        Ok(res)
    }

    fn eval_while_statement(
        &mut self,
        condition: Box<Node>,
        body: Box<Node>,
        env_id: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let condition_resolved = self.resolve(condition.clone(), env_id)?;
        let condition_innner = condition_resolved.lock().unwrap();
        if condition_innner.kind() != ValueType::Boolean {
            bail!(InterpreterError::InvalidCondition(dyn_clone::clone_box(
                &**condition_innner
            )))
        }

        let mut boolean = cast_value::<BoolValue>(&condition_innner).unwrap();
        while boolean.value() {
            self.resolve(body.clone(), env_id)?;

            let condition_resolved = self.resolve(condition.clone(), env_id)?;
            let condition_innner = condition_resolved.lock().unwrap();
            if condition_innner.kind() != ValueType::Boolean {
                bail!(InterpreterError::InvalidCondition(dyn_clone::clone_box(
                    &**condition_innner
                )))
            }
            boolean = cast_value::<BoolValue>(&condition_innner).unwrap();
        }

        return Ok(Arc::new(Mutex::new(Box::new(NullValue::default()))));
    }

    fn eval_class_declaration(
        &mut self,
        name: String,
        super_class: Option<Box<Node>>,
        body: Vec<Box<Node>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut class = ClassValue::default();
        class.name = name.clone();
        match super_class {
            Some(id) => {
                let resolved = self.resolve(id, env)?;
                let target = resolved.lock().unwrap();
                if target.kind() == ValueType::Class {
                    let target_class = cast_value::<ClassValue>(&target).unwrap();
                    class.copy_properties(&target_class);
                    class.copy_methods(&target_class);
                    class.super_class = Some(target_class);
                }
            }
            None => {}
        };
        for class_stmt in body {
            match *class_stmt {
                Node::PropertyDefinition(name, value, is_static) => {
                    class.insert_property(ClassProperty {
                        value: self.resolve(value, env)?,
                        name,
                        is_static,
                    })
                }
                Node::MethodDefinition(name, method_params, body, is_static) => {
                    let mut params: Vec<ClassMethodParameter> = vec![];
                    for param in method_params {
                        match *param {
                            Node::Identifier(name) => params.push(ClassMethodParameter {
                                name,
                                default_value: None,
                            }),
                            // TODO: Add support for default value (Assignment Expression)
                            _ => bail!(InterpreterError::UnexpectedNode(dyn_clone::clone_box(
                                &*param
                            ))),
                        }
                    }
                    class.insert_method(ClassMethod {
                        name,
                        is_static,
                        args: params,
                        body: body,
                    })
                }
                _ => {}
            };
        }
        let scope_c = SCOPE_STATE.clone();
        let mut scope_state = scope_c.lock().unwrap();
        let scope = match scope_state.get_scope_mut(env) {
            Some(scope) => scope,
            // TODO: return correct error
            None => bail!(InterpreterError::InvalidFunctionEnvironment(env)),
        };
        let result =
            scope.declare_variable(name, Arc::new(Mutex::new(Box::new(class.clone()))), true)?;
        drop(scope_state);
        drop(scope_c);
        Ok(result)
    }

    fn eval_member_expression(
        &mut self,
        object: Box<Node>,
        property: Box<Node>,
        computed: bool,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let object = self.resolve(object, env)?;

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
        if property_inner.kind() != ValueType::String {
            bail!(InterpreterError::UnsupportedValue(property))
        }

        let key = cast_value::<StringValue>(&property_inner).unwrap().value();

        let obj = object.clone();
        let object_inner = obj.lock().unwrap();
        let value = match object_inner.kind() {
            ValueType::Object => dyn_clone::clone_box(&**object_inner),
            ValueType::Class => {
                let class = cast_value::<ClassValue>(&object_inner).unwrap();
                let class_prop = class.get_static_property(key.clone());
                if class_prop.is_none() {
                    let class_method = class.get_static_method(key.clone());
                    if let Some(class_method) = class_method {
                        let mut temp_map: HashMap<Key, Value> = HashMap::new();
                        let func = FunctionValue::new(
                            class_method.name,
                            class_method
                                .args
                                .iter()
                                .map(|arg| FunctionParameter {
                                    name: arg.name.clone(),
                                    default_value: arg.default_value.clone(),
                                })
                                .collect(),
                            // TODO: look if correct env
                            env,
                            class_method.body,
                        );
                        temp_map.insert(key.clone(), Arc::new(Mutex::new(Box::new(func))));
                        Box::new(ObjectValue::from(temp_map))
                    } else {
                        bail!(InterpreterError::UnresolvedProperty(key.clone()))
                    }
                } else {
                    let mut temp_map: HashMap<Key, Value> = HashMap::new();
                    temp_map.insert(key.clone(), class_prop.unwrap().value);
                    Box::new(ObjectValue::from(temp_map))
                }
            }
            ValueType::String => {
                let string_value = cast_value::<StringValue>(&object_inner).unwrap();
                get_string_object(&string_value)
            }
            ValueType::Integer | ValueType::Decimal => {
                let number = dyn_clone::clone_box(&**object_inner);
                get_number_object(&number)
            }
            _ => bail!(InterpreterError::UnexpectedValue(dyn_clone::clone_box(
                &**object_inner
            ))),
        };

        let object = cast_value::<ObjectValue>(&value).unwrap();

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

    fn eval_class_call(
        &mut self,
        object: Box<Node>,
        obj_val: Box<dyn RuntimeValue>,
        callee_inner: Box<dyn RuntimeValue>,
        args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let obj = cast_value::<ObjectValue>(&obj_val).unwrap();
        if callee_inner.kind() != ValueType::Function {
            bail!(InterpreterError::InvalidFunctionCallee(Arc::new(
                Mutex::new(callee_inner)
            )))
        }
        let func = cast_value::<FunctionValue>(&callee_inner).unwrap();

        let mut scope_state = SCOPE_STATE.lock().unwrap();
        let env_id = scope_state.create_environment(Some(func.declaration_env));
        let scope = scope_state.get_scope_mut(env_id).unwrap();
        scope.declare_variable("self".into(), Arc::new(Mutex::new(obj)), true)?;
        // TODO: validate provided arguments
        for (index, parameter) in func.parameters.iter().enumerate() {
            let value = args.get(index).unwrap().clone();
            scope.declare_variable(parameter.name.clone(), value, true)?;
        }
        drop(scope_state);
        let value = self.resolve(func.body, env_id)?;
        let scope_state = SCOPE_STATE.lock().unwrap();
        let scope = scope_state.get_scope(env_id).unwrap();
        let new_self = scope.lookup_variable("self".into(), &scope_state)?;
        drop(scope_state);
        let mut assigne = Some(object.clone());
        let mut assign_value = new_self;
        while let Some(ref assignee) = assigne {
            match *dyn_clone::clone_box(&**assignee) {
                Node::Identifier(var_name) => {
                    self.assign_variable(var_name, assign_value.clone(), env, true)?;
                    assigne = None;
                }
                Node::MemberExpression(parent, current, computed) => {
                    let prop: Arc<Mutex<Box<dyn RuntimeValue>>> = if computed {
                        self.resolve(current, env)?
                    } else {
                        match *current {
                            Node::Identifier(value) => {
                                Arc::new(Mutex::new(Box::new(StringValue::from(value))))
                            }
                            _ => bail!(InterpreterError::UnsupportedNode(current)),
                        }
                    };
                    let prop_inner = prop.lock().unwrap();
                    if prop_inner.kind() != ValueType::String {
                        bail!(InterpreterError::UnsupportedValue(Arc::new(Mutex::new(
                            dyn_clone::clone_box(&**prop_inner)
                        ))))
                    }
                    let prop_str = cast_value::<StringValue>(&prop_inner).unwrap();

                    let obj_val = self.resolve(parent.clone(), env)?;
                    let obj_inner = obj_val.lock().unwrap();
                    if obj_inner.kind() == ValueType::Object {
                        let mut obj = cast_value::<ObjectValue>(&obj_inner).unwrap();
                        obj.assign_property(prop_str.value(), assign_value.clone());
                        assigne = Some(parent);
                        assign_value = Arc::new(Mutex::new(obj));
                    } else {
                        // TODO: look if this correct to look only for object values
                        bail!(InterpreterError::UnexpectedNode(object.clone()))
                    }
                }
                node => bail!(InterpreterError::UnexpectedNode(Box::new(node))),
            };
        }
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
        if let Node::MemberExpression(object, property, _computed) =
            *dyn_clone::clone_box(&*callee.clone())
        {
            let object_value = self.resolve(object.clone(), env)?;
            let obj = object_value.lock().unwrap();
            if obj.kind() == ValueType::Object {
                let class_obj = cast_value::<ObjectValue>(&obj).unwrap();
                if let Node::Identifier(func_name) = *property {
                    let calle = class_obj.get_property(func_name.clone());
                    if calle.is_none() {
                        bail!(InterpreterError::UnresolvedProperty(func_name.clone()))
                    }
                    let calle_res = calle.unwrap();
                    let fn_callee = calle_res.lock().unwrap();
                    if obj.kind() == ValueType::Object && fn_callee.kind() == ValueType::Function {
                        return Ok(self.eval_class_call(
                            object,
                            dyn_clone::clone_box(&**obj),
                            dyn_clone::clone_box(&**fn_callee),
                            args,
                            env,
                        )?);
                    }
                }
            }
        }
        let fn_callee = self.resolve(callee, env)?;

        let fn_calle_c = fn_callee.clone();
        let fn_callee_box = fn_calle_c.lock().unwrap();
        let result: Arc<Mutex<Box<dyn RuntimeValue>>> = match fn_callee_box.kind().clone() {
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
                // TODO: validate provided arguments
                for (index, parameter) in func_c.parameters.iter().enumerate() {
                    let value = args.get(index).unwrap().clone();
                    scope.declare_variable(parameter.name.clone(), value, true)?;
                }
                drop(scope_state);
                let value = self.resolve(func_c.body, env_id)?;
                value
                // Arc::new(Mutex::new(Box::new(NullValue::default())))
            }
            ValueType::Class => {
                let class = match fn_callee_box.into_any().downcast::<ClassValue>() {
                    Ok(class) => class,
                    Err(_) => bail!(InterpreterError::UnsupportedValue(fn_callee.clone())),
                };
                drop(fn_callee_box);
                drop(fn_callee);
                let methods = class.methods;
                let properties = class.properties;

                let mut instance_map: HashMap<Key, Value> = HashMap::new();
                for property in properties.iter().filter(|prop| !prop.is_static) {
                    instance_map.insert(property.name.clone(), property.value.clone());
                }

                for (method_name, method) in
                    methods.clone().iter().filter(|method| !method.1.is_static)
                {
                    if method_name == "__new__" {
                        continue;
                    }
                    let function = FunctionValue::new(
                        method_name.clone(),
                        method
                            .args
                            .iter()
                            .map(|arg| {
                                FunctionParameter::new(arg.name.clone(), arg.default_value.clone())
                            })
                            .collect(),
                        env,
                        method.clone().body,
                    );
                    instance_map.insert(
                        method_name.clone(),
                        Arc::new(Mutex::new(Box::new(function))),
                    );
                }

                if methods.contains_key(&String::from("__new__")) {
                    let constructor = methods.get(&String::from("__new__")).unwrap();
                    let init_args = constructor.args.clone();
                    // don't allow default value for first args
                    // e.g. _(arg1 = null, arg2, arg3) - invalid
                    // e.g. _(arg1, arg2 = null, arg3) - invalid
                    //      _(arg1, arg2, arg3 = null) - valid
                    for (index, arg) in init_args.iter().enumerate() {
                        if arg.default_value.is_some() && index + 1 < init_args.len() {
                            bail!(InterpreterError::InvalidDefaultParameter(arg.name.clone()))
                        }
                    }
                    let required_args = init_args
                        .iter()
                        .filter(|arg| arg.default_value.is_none())
                        .count();
                    if args.len() < required_args {
                        bail!(InterpreterError::InvalidParameterCount(
                            required_args,
                            args.len()
                        ))
                    }

                    let obj = ObjectValue::from(instance_map);
                    let mut scope_state = SCOPE_STATE.lock().unwrap();
                    // TODO: look if we use correct environment
                    let env_id = scope_state.create_environment(Some(env));
                    let scope = scope_state.get_scope_mut(env_id).unwrap();
                    for (index, arg) in args.iter().enumerate() {
                        if let Some(init_arg) = init_args.get(index) {
                            scope.declare_variable(init_arg.name.clone(), arg.clone(), true)?;
                        }
                    }
                    scope.declare_variable(
                        "self".into(),
                        Arc::new(Mutex::new(Box::new(obj.clone()))),
                        true,
                    )?;
                    if let Some(super_class) = class.super_class {
                        let super_constructor = super_class.methods.get("__new__");
                        if super_constructor.is_some() {
                            let super_constructor = super_constructor.unwrap();
                            let super_init_args = super_constructor.args.clone();

                            // don't allow default value for first args
                            // e.g. _(arg1 = null, arg2, arg3) - invalid
                            // e.g. _(arg1, arg2 = null, arg3) - invalid
                            //      _(arg1, arg2, arg3 = null) - valid
                            for (index, arg) in super_init_args.iter().enumerate() {
                                if arg.default_value.is_some() && index + 1 < super_init_args.len()
                                {
                                    bail!(InterpreterError::InvalidDefaultParameter(
                                        arg.name.clone()
                                    ))
                                }
                            }

                            let super_func = FunctionValue::new(
                                "super".into(),
                                super_init_args
                                    .iter()
                                    .map(|arg| FunctionParameter {
                                        name: arg.name.clone(),
                                        default_value: arg.default_value.clone(),
                                    })
                                    .collect(),
                                env_id,
                                super_constructor.body.clone(),
                            );

                            scope.declare_variable(
                                "super".into(),
                                Arc::new(Mutex::new(Box::new(super_func))),
                                true,
                            )?;
                        }
                    }
                    drop(scope_state);
                    self.resolve(constructor.body.clone(), env_id)?;
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env_id).unwrap();
                    let value = scope.lookup_variable("self".into(), &scope_state)?;
                    value
                } else {
                    Arc::new(Mutex::new(Box::new(ObjectValue::from(instance_map))))
                }
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
        ignore_constant: bool,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let mut scope_state = SCOPE_STATE.lock().unwrap();
        Ok(scope_state.assign_variable(name, value, env, ignore_constant)?)
    }

    fn eval_assignment_expression(
        &mut self,
        left: Box<Node>,
        operator: AssignmentOperator,
        right: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        if let Node::MemberExpression(object, property, computed) = *left {
            let obj_val = self.resolve(object.clone(), env)?;
            let obj_inner = obj_val.lock().unwrap();
            match obj_inner.kind() {
                ValueType::Object => {
                    let mut obj = cast_value::<ObjectValue>(&obj_inner).unwrap();
                    let prop: Arc<Mutex<Box<dyn RuntimeValue>>> = if computed {
                        self.resolve(property, env)?
                    } else {
                        match *property {
                            Node::Identifier(value) => {
                                Arc::new(Mutex::new(Box::new(StringValue::from(value))))
                            }
                            _ => bail!(InterpreterError::UnsupportedNode(property)),
                        }
                    };
                    let prop_inner = prop.lock().unwrap();
                    if prop_inner.kind() != ValueType::String {
                        bail!(InterpreterError::UnsupportedValue(prop.clone()))
                    }
                    let prop_name = cast_value::<StringValue>(&prop_inner).unwrap().value();
                    let obj_map = obj.map();
                    if operator == AssignmentOperator::Equals {
                        let right_val = self.resolve(right.clone(), env)?;
                        obj.assign_property(prop_name, right_val);
                        if let Node::Identifier(object_name) = *object {
                            return Ok(self.assign_variable(
                                object_name,
                                Arc::new(Mutex::new(obj)),
                                env,
                                true,
                            )?);
                        } else {
                            return Ok(self.eval_assignment_expression(
                                object,
                                operator,
                                right.clone(),
                                env,
                            )?);
                        }
                    }
                    let previous_value_opt = obj_map.get(&prop_name);
                    if let Some(previous_value) = previous_value_opt {
                        let left = self.convert_value_to_node(dyn_clone::clone_box(
                            &**previous_value.lock().unwrap(),
                        ));
                        let binary_op = match operator {
                            AssignmentOperator::Addition => BinaryOperator::Plus,
                            AssignmentOperator::Division => BinaryOperator::Divide,
                            AssignmentOperator::Modulation => BinaryOperator::Modulo,
                            AssignmentOperator::Multiplication => BinaryOperator::Multiply,
                            AssignmentOperator::Subtraction => BinaryOperator::Minus,
                            AssignmentOperator::Equals => {
                                panic!("unexpected equals assignment operator")
                            }
                        };
                        let binary = Node::BinaryExpression(left, binary_op, right.clone());
                        let value = self.resolve(Box::new(binary), env)?;
                        obj.assign_property(prop_name, value);
                        if let Node::Identifier(object_name) = *object {
                            Ok(self.assign_variable(
                                object_name,
                                Arc::new(Mutex::new(obj)),
                                env,
                                true,
                            )?)
                        } else {
                            self.eval_assignment_expression(object, operator, right.clone(), env)
                        }
                    } else {
                        bail!(InterpreterError::UnresolvedProperty(prop_name))
                    }
                }
                ValueType::Class => {
                    let mut class = cast_value::<ClassValue>(&obj_inner).unwrap();
                    let prop: Arc<Mutex<Box<dyn RuntimeValue>>> = if computed {
                        self.resolve(property, env)?
                    } else {
                        match *property {
                            Node::Identifier(value) => {
                                Arc::new(Mutex::new(Box::new(StringValue::from(value))))
                            }
                            _ => bail!(InterpreterError::UnsupportedNode(property)),
                        }
                    };
                    let prop_inner = prop.lock().unwrap();
                    if prop_inner.kind() != ValueType::String {
                        bail!(InterpreterError::UnsupportedValue(prop.clone()))
                    }
                    let prop_name = cast_value::<StringValue>(&prop_inner).unwrap().value();
                    if operator == AssignmentOperator::Equals {
                        let right_val = self.resolve(right.clone(), env)?;
                        class.insert_property(ClassProperty {
                            is_static: true,
                            name: prop_name,
                            value: right_val,
                        });
                        if let Node::Identifier(object_name) = *object {
                            return Ok(self.assign_variable(
                                object_name,
                                Arc::new(Mutex::new(class)),
                                env,
                                true,
                            )?);
                        } else {
                            return Ok(self.eval_assignment_expression(
                                object,
                                operator,
                                right.clone(),
                                env,
                            )?);
                        }
                    }
                    let previous_value_opt = class.get_static_property(prop_name.clone());
                    if let Some(previous_value) = previous_value_opt {
                        let left = self.convert_value_to_node(dyn_clone::clone_box(
                            &**previous_value.value.lock().unwrap(),
                        ));
                        let binary_op = match operator {
                            AssignmentOperator::Addition => BinaryOperator::Plus,
                            AssignmentOperator::Division => BinaryOperator::Divide,
                            AssignmentOperator::Modulation => BinaryOperator::Modulo,
                            AssignmentOperator::Multiplication => BinaryOperator::Multiply,
                            AssignmentOperator::Subtraction => BinaryOperator::Minus,
                            AssignmentOperator::Equals => {
                                panic!("unexpected equals assignment operator")
                            }
                        };
                        let binary = Node::BinaryExpression(left, binary_op, right.clone());
                        let value = self.resolve(Box::new(binary), env)?;
                        class.insert_property(ClassProperty {
                            name: prop_name,
                            is_static: true,
                            value,
                        });
                        if let Node::Identifier(object_name) = *object {
                            Ok(self.assign_variable(
                                object_name,
                                Arc::new(Mutex::new(class)),
                                env,
                                true,
                            )?)
                        } else {
                            self.eval_assignment_expression(object, operator, right.clone(), env)
                        }
                    } else {
                        bail!(InterpreterError::UnresolvedProperty(prop_name))
                    }
                }
                _ => bail!(InterpreterError::InvalidValue(
                    dyn_clone::clone_box(&**obj_inner),
                    "object or class".to_string()
                )),
            }
        } else if let Node::Identifier(variable_name) = *left {
            let value = match operator {
                AssignmentOperator::Equals => {
                    let right = self.resolve(right, env)?;
                    self.assign_variable(variable_name.clone(), right, env, false)?
                }
                assignment_operator => {
                    let scope_state = SCOPE_STATE.lock().unwrap();
                    let scope = scope_state.get_scope(env).unwrap();
                    let previous_value =
                        scope.lookup_variable(variable_name.clone(), &scope_state)?;
                    let left = self.convert_value_to_node(dyn_clone::clone_box(
                        &**previous_value.lock().unwrap(),
                    ));
                    let operator = match assignment_operator {
                        AssignmentOperator::Addition => BinaryOperator::Plus,
                        AssignmentOperator::Division => BinaryOperator::Divide,
                        AssignmentOperator::Modulation => BinaryOperator::Modulo,
                        AssignmentOperator::Multiplication => BinaryOperator::Multiply,
                        AssignmentOperator::Subtraction => BinaryOperator::Minus,
                        AssignmentOperator::Equals => {
                            panic!("unexpected equals assignment operator")
                        }
                    };
                    let binary = Node::BinaryExpression(left, operator, right);
                    let value = self.resolve(Box::new(binary), env)?;
                    drop(scope_state);
                    self.assign_variable(variable_name.clone(), value, env, false)?
                }
            };
            return Ok(value);
        } else {
            bail!(InterpreterError::InvalidAssignFactor(left))
        }
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
                )))));
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

    fn eval_string_binary_expression(
        &mut self,
        left: Arc<Mutex<Box<dyn RuntimeValue>>>,
        right: Arc<Mutex<Box<dyn RuntimeValue>>>,
        operator: BinaryOperator,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        let left = left.lock().unwrap();
        let right = right.lock().unwrap();

        let left_str = cast_value::<StringValue>(&left).unwrap();
        let right_str = cast_value::<StringValue>(&right).unwrap();

        match operator {
            BinaryOperator::IsEquals => {
                return Ok(Arc::new(Mutex::new(Box::new(BoolValue::from(
                    left_str.value() == right_str.value(),
                )))))
            }
            // TODO: maybe adding string concatination as Plus?
            // FIXME: throw appropriate error here
            _ => bail!(InterpreterError::InvalidCondition(dyn_clone::clone_box(
                &**left
            ))),
        }
    }

    fn eval_binary_expression(
        &mut self,
        node: Box<Node>,
        env: EnvironmentId,
    ) -> Result<Arc<Mutex<Box<dyn RuntimeValue>>>, InterpreterError> {
        if let Node::BinaryExpression(left, operator, right_node) = *node {
            let left = self.resolve(left, env)?;
            let right = self.resolve(right_node, env)?;

            let left_inn = left.lock().unwrap();
            let left_kind = dyn_clone::clone_box(&**left_inn).kind();
            let right_inn = right.lock().unwrap();
            let right_kind = dyn_clone::clone_box(&**right_inn).kind();
            drop(left_inn);
            drop(right_inn);

            // TODO: add support for data type conversion, e.g. number.toString() + String
            if (left_kind == ValueType::Integer || left_kind == ValueType::Decimal)
                && (right_kind == ValueType::Integer || right_kind == ValueType::Decimal)
            {
                let value = self.eval_numeric_binary_expression(left, right, operator)?;
                return Ok(value);
            } else if left_kind == ValueType::String && right_kind == ValueType::String {
                return Ok(self.eval_string_binary_expression(left, right, operator)?);
            }

            return Ok(Arc::new(Mutex::new(Box::new(NullValue::default()))));
        } else {
            bail!(InterpreterError::UnexpectedNode(node))
        }
    }
}
