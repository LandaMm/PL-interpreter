use std::fs;

use pl_ast::Parser;
use pl_interpreter::Interpreter;

fn main() {
    let source = fs::read_to_string("test/main.pl").unwrap();
    let mut parser = Parser::from_source(source);
    let ast = parser.produce_ast().unwrap();
    let interpreter = Interpreter::new();
    let result = interpreter.evaluate(Box::new(ast)).unwrap();
    println!("result: {:?}", result);
}
