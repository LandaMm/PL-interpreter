use std::{
    fs,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use pl_ast::Parser;
use pl_interpreter::{
    BoolValue, ClosureType, Environment, IntegerValue, Interpreter, InterpreterError,
    NativeFnValue, NullValue, RuntimeValue, WithFnCall, SCOPE_STATE,
};

fn mk_runtime_value(value: Box<dyn RuntimeValue>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    Arc::new(Mutex::new(value))
}

fn mk_native_fn(func: ClosureType) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    let with_call = WithFnCall::new(func);
    Arc::new(Mutex::new(Box::new(NativeFnValue::new(with_call))))
}

fn native_print_function(
    args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    println!(
        "{:?}",
        args.into_iter()
            .map(|arg| {
                let val = arg.lock().unwrap();
                dyn_clone::clone_box(&**val)
            })
            .collect::<Vec<Box<dyn RuntimeValue>>>()
    );
    mk_runtime_value(Box::new(NullValue::default()))
}

fn native_get_time(
    args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    mk_runtime_value(Box::new(IntegerValue::from(
        since_the_epoch.as_millis() as isize
    )))
}

fn main() -> Result<(), InterpreterError> {
    let source = fs::read_to_string("test/main.pl").unwrap();
    let mut parser = Parser::from_source(source);
    let ast = parser.produce_ast().unwrap();
    let mut interpreter = Interpreter::new();

    let mut environment = Environment::new(None);
    environment.declare_variable(
        "true".to_string(),
        mk_runtime_value(Box::new(BoolValue::from(true))),
        true,
    )?;
    environment.declare_variable(
        "false".to_string(),
        mk_runtime_value(Box::new(BoolValue::from(false))),
        true,
    )?;
    environment.declare_variable(
        "null".to_string(),
        mk_runtime_value(Box::new(NullValue::default())),
        true,
    )?;

    environment.declare_variable(
        "print".to_string(),
        mk_native_fn(Arc::new(Mutex::new(native_print_function))),
        true,
    )?;
    environment.declare_variable(
        "time".to_owned(),
        mk_native_fn(Arc::new(Mutex::new(native_get_time))),
        true,
    )?;

    let env_id = SCOPE_STATE.lock().unwrap().append_environment(environment);
    // println!("environment initiated: {env_id}");

    let result = match interpreter.evaluate(Box::new(ast), env_id) {
        Ok(res) => res,
        Err(err) => panic!("{}", err),
    };

    println!(
        "\n\n-----------------------\n{:#?}",
        SCOPE_STATE.lock().unwrap()
    );

    // println!("result: {:?}", result);

    Ok(())
}
