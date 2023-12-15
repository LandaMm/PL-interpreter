use std::{fs, thread};

use pl_ast::Parser;
use pl_interpreter::{setup_environment, Interpreter};

fn main() {
    thread::Builder::new()
        .stack_size(128 * 1024 * 1024)
        .spawn(|| {
            let source = fs::read_to_string("test/main.pl").unwrap();
            let mut parser = Parser::from_source(source);
            let ast = parser.produce_ast().unwrap();
            let mut interpreter = Box::new(Interpreter::new());

            let env_id = setup_environment();

            match interpreter.run(Box::new(ast), env_id, false) {
                Ok(res) => res,
                Err(err) => panic!("{}", err),
            };

            // println!(
            //     "\n\n-----------------------\n{:#?}",
            //     SCOPE_STATE.lock().unwrap()
            // );

            // println!("result: {:?}", result);
        })
        .unwrap()
        .join()
        .unwrap();
}

// fn loop_(cb: fn(usize), index: usize, limit: usize) {
//     if index < limit {
//         cb(index);
//         loop_(cb, index + 1, limit);
//     }
// }

// fn func(index: usize) {
//     println!("{index}");
// }

// fn main() {
//     loop_(func, 0, 1000000);
// }
