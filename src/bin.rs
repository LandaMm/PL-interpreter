use std::{env, fs, thread};

use pl_ast::Parser;
use pl_interpreter::{setup_default_environment, Interpreter};

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let file_name = if args.len() > 1 {
        args.get(1).unwrap().to_owned()
    } else {
        "test/main.amr".to_string()
    };

    thread::Builder::new()
        .stack_size(1024 * 1024)
        .spawn(|| {
            let source = fs::read_to_string(file_name).unwrap();
            let mut parser = Parser::from_source(source);
            let ast = parser.produce_ast().unwrap();
            let mut interpreter = Box::new(Interpreter::new());

            let env_id = setup_default_environment();

            match interpreter.run(Box::new(ast), env_id) {
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
