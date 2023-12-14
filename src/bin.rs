use std::fs;

use pl_ast::Parser;
use pl_interpreter::{setup_environment, Interpreter};

fn main() {
    let source = fs::read_to_string("test/main.pl").unwrap();
    let mut parser = Parser::from_source(source);
    let ast = parser.produce_ast().unwrap();
    let mut interpreter = Interpreter::new();

    let env_id = setup_environment();

    match interpreter.evaluate(Box::new(ast), env_id) {
        Ok(res) => res,
        Err(err) => panic!("{}", err),
    };

    // println!(
    //     "\n\n-----------------------\n{:#?}",
    //     SCOPE_STATE.lock().unwrap()
    // );

    // println!("result: {:?}", result);
}
