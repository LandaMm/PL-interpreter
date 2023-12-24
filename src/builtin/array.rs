use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    cast_value, macros::bail, ArrayValue, ClassValue, ClosureType, EnvironmentId,
    FunctionParameter, FunctionValue, IntegerValue, InterpreterError, Key, NullValue, ObjectValue,
    RuntimeValue, Value, ValueType, SCOPE_STATE,
};

use super::{mk_native_fn, mk_runtime_value};

pub fn get_by_index(value: ArrayValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.is_empty() {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let arg = args.get(0).unwrap();
            let arg_val = arg
                .lock()
                .expect("get_by_index: failed to get first argument");

            if arg_val.kind() != ValueType::Integer {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let index = cast_value::<IntegerValue>(&arg_val).unwrap();

            if index.value() < 0 {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            if value.value().len() <= index.value() as usize {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            value.value().get(index.value() as usize).unwrap().clone()
        },
    ))
}

pub fn merge(value: ArrayValue) -> ClosureType {
    Arc::new(Mutex::new(
        move |args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>| {
            if args.is_empty() {
                return mk_runtime_value(Box::new(NullValue::default()));
            }

            let mut new_array = value.clone();

            for arg in args {
                new_array.append_element(arg)
            }

            mk_runtime_value(Box::new(new_array))
        },
    ))
}

pub fn get_array_object(
    array: &ArrayValue,
    key: &String,
    env: EnvironmentId,
) -> Result<Box<ObjectValue>, InterpreterError> {
    let mut map: HashMap<Key, Value> = HashMap::new();

    map.insert(
        "get".into(),
        mk_native_fn("array.get".into(), get_by_index(array.clone())),
    );

    map.insert(
        "merge".into(),
        mk_native_fn("array.append".into(), merge(array.clone())),
    );

    map.insert(
        "length".into(),
        mk_runtime_value(Box::new(IntegerValue::from(array.value().len() as isize))),
    );

    let scope_state = SCOPE_STATE
        .lock()
        .expect("get_array_object: failed to get immutable scope state");
    let scope = match scope_state.get_scope(env) {
        Some(scope) => scope,
        None => bail!(InterpreterError::UnresolvedEnvironment(env)),
    };

    let array_value = scope.lookup_variable_safe("Array".to_string(), &scope_state);

    if let Some(array_class_val) = array_value {
        let array_class = array_class_val
            .lock()
            .expect("get_array_object: failed to get Array class");
        if array_class.kind() == ValueType::Class {
            let arr_prot = cast_value::<ClassValue>(&array_class).unwrap();
            let method = arr_prot
                .methods
                .iter()
                .find(|method| method.0 == key && !method.1.is_static);
            if let Some((m_name, method)) = method {
                let func = FunctionValue::new(
                    m_name.clone(),
                    method
                        .args
                        .iter()
                        .map(|arg| FunctionParameter {
                            name: arg.name.clone(),
                            default_value: arg.default_value.clone(),
                        })
                        .collect(),
                    env,
                    method.body.clone(),
                );
                map.insert(key.clone(), Arc::new(Mutex::new(Box::new(func))));
            }
        } else {
            bail!(InterpreterError::UnexpectedValue(dyn_clone::clone_box(
                &**array_class
            )))
        }
    }

    Ok(Box::new(ObjectValue::from(map)))
}
