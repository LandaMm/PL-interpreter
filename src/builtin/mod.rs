mod cast_value;
mod math;
mod number;
mod print;
mod string;
mod stringify;
mod time;
mod type_of;

use crate::{
    BoolValue, ClosureType, Environment, EnvironmentId, NativeFnValue, NullValue, ObjectValue,
    RuntimeValue, WithFnCall, SCOPE_STATE,
};
use std::sync::{Arc, Mutex};

pub use cast_value::*;
pub use math::*;
pub use number::*;
pub use print::*;
pub use string::*;
pub use stringify::*;
pub use time::*;
pub use type_of::*;

fn mk_runtime_value(value: Box<dyn RuntimeValue>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    Arc::new(Mutex::new(value))
}

fn mk_native_fn(name: String, func: ClosureType) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    let with_call = WithFnCall::new(func);
    Arc::new(Mutex::new(Box::new(NativeFnValue::new(name, with_call))))
}

pub fn setup_default_environment() -> EnvironmentId {
    // basic constants
    let mut environment = Environment::new(None);
    environment
        .declare_variable(
            "true".to_string(),
            mk_runtime_value(Box::new(BoolValue::from(true))),
            true,
        )
        .unwrap();
    environment
        .declare_variable(
            "false".to_string(),
            mk_runtime_value(Box::new(BoolValue::from(false))),
            true,
        )
        .unwrap();
    environment
        .declare_variable(
            "null".to_string(),
            mk_runtime_value(Box::new(NullValue::default())),
            true,
        )
        .unwrap();

    // native print
    environment
        .declare_variable(
            "print".to_string(),
            mk_native_fn(
                "print".to_string(),
                Arc::new(Mutex::new(native_print_function)),
            ),
            true,
        )
        .unwrap();

    // native time
    environment
        .declare_variable(
            "time".to_string(),
            mk_native_fn("time".to_string(), Arc::new(Mutex::new(native_get_time))),
            true,
        )
        .unwrap();

    // native math object
    environment
        .declare_variable(
            "math".to_string(),
            mk_runtime_value(Box::new(ObjectValue::from(get_math()))),
            true,
        )
        .unwrap();

    environment
        .declare_variable(
            "type_of".to_string(),
            mk_native_fn("type_of".into(), Arc::new(Mutex::new(native_type_of))),
            true,
        )
        .unwrap();

    let env_id = SCOPE_STATE.lock().unwrap().append_environment(environment);

    env_id
}
