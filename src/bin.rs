use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

use pl_ast::Parser;
use pl_interpreter::{
    BoolValue, Environment, IntegerValue, Interpreter, InterpreterError, NativeFnValue, NullValue,
    RuntimeValue,
};

fn native_print_function(
    args: Vec<Box<dyn RuntimeValue>>,
    scope: &Environment,
) -> Box<dyn RuntimeValue> {
    println!("native_print: {args:?}");
    Box::new(NullValue::default())
}

fn native_get_time(args: Vec<Box<dyn RuntimeValue>>, scope: &Environment) -> Box<dyn RuntimeValue> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    Box::new(IntegerValue::from(since_the_epoch.as_millis() as isize))
}

fn main() -> Result<(), InterpreterError> {
    let source = fs::read_to_string("test/main.pl").unwrap();
    let mut parser = Parser::from_source(source);
    let ast = parser.produce_ast().unwrap();
    let interpreter = Interpreter::new();

    let mut environment = Environment::new(None);
    environment.declare_variable("true".to_string(), Box::new(BoolValue::from(true)), true)?;
    environment.declare_variable("false".to_string(), Box::new(BoolValue::from(false)), true)?;
    environment.declare_variable("null".to_string(), Box::new(NullValue::default()), true)?;

    environment.declare_variable(
        "print".to_string(),
        Box::new(NativeFnValue::new(native_print_function)),
        true,
    )?;
    environment.declare_variable(
        "time".to_owned(),
        Box::new(NativeFnValue::new(native_get_time)),
        true,
    )?;

    let result = match interpreter.evaluate(Box::new(ast), &mut environment) {
        Ok(res) => res,
        Err(err) => panic!("{}", err),
    };

    // println!("result: {:?}", result);

    Ok(())
}
