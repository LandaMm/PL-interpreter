use std::fs;

use pl_ast::Parser;
use pl_interpreter::{BoolValue, Environment, Interpreter, InterpreterError, NullValue};

fn main() -> Result<(), InterpreterError> {
    let source = fs::read_to_string("test/main.pl").unwrap();
    let mut parser = Parser::from_source(source);
    let ast = parser.produce_ast().unwrap();
    let interpreter = Interpreter::new();

    let mut environment = Environment::new(None);
    environment.declare_variable("true".to_string(), Box::new(BoolValue::from(true)))?;
    environment.declare_variable("false".to_string(), Box::new(BoolValue::from(false)))?;
    environment.declare_variable("null".to_string(), Box::new(NullValue::default()))?;

    let result = match interpreter.evaluate(Box::new(ast), &mut environment) {
        Ok(res) => res,
        Err(err) => panic!("{}", err),
    };

    println!("result: {:?}", result);

    Ok(())
}
