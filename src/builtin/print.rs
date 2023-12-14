use std::sync::{Arc, Mutex};

use crate::{
    builtin::{mk_runtime_value, stringify},
    NullValue, RuntimeValue,
};

pub fn native_print_function(
    args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    println!(
        "{}",
        args.into_iter()
            .map(|arg| {
                let val = arg.lock().unwrap();
                let cloned = dyn_clone::clone_box(&**val);
                stringify(cloned)
            })
            .collect::<Vec<String>>()
            .join(" ")
    );
    mk_runtime_value(Box::new(NullValue::default()))
}
