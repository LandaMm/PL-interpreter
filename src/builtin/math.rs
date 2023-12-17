use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{cast_value, DecimalValue, Key, NullValue, RuntimeValue, Value, ValueType};

use super::{mk_native_fn, mk_runtime_value};

fn random(_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    let mut rng = rand::thread_rng();

    let num = rng.gen::<f64>();

    mk_runtime_value(Box::new(DecimalValue::from(num)))
}

fn floor(args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args.get(0).unwrap();
    let value = arg.lock().unwrap();

    if value.kind() != ValueType::Decimal && value.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if value.kind() == ValueType::Decimal {
        let decimal = cast_value::<DecimalValue>(&value).unwrap();
        return mk_runtime_value(Box::new(DecimalValue::from(decimal.value().floor())));
    }

    // only integer case left
    return arg.clone();
}

fn ceil(args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args.get(0).unwrap();
    let value = arg.lock().unwrap();

    if value.kind() != ValueType::Decimal && value.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if value.kind() == ValueType::Decimal {
        let decimal = cast_value::<DecimalValue>(&value).unwrap();
        return mk_runtime_value(Box::new(DecimalValue::from(decimal.value().ceil())));
    }

    // only integer case left
    return arg.clone();
}

fn trunc(args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args.get(0).unwrap();
    let value = arg.lock().unwrap();

    if value.kind() != ValueType::Decimal && value.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if value.kind() == ValueType::Decimal {
        let decimal = cast_value::<DecimalValue>(&value).unwrap();
        return mk_runtime_value(Box::new(DecimalValue::from(decimal.value().trunc())));
    }

    // only integer case left
    return arg.clone();
}

fn round(args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args.get(0).unwrap();
    let value = arg.lock().unwrap();

    if value.kind() != ValueType::Decimal && value.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if value.kind() == ValueType::Decimal {
        let decimal = cast_value::<DecimalValue>(&value).unwrap();
        return mk_runtime_value(Box::new(DecimalValue::from(decimal.value().round())));
    }

    // only integer case left
    return arg.clone();
}

pub fn get_math() -> HashMap<Key, Value> {
    let mut map: HashMap<Key, Value> = HashMap::new();

    map.insert(
        "PI".to_string(),
        mk_runtime_value(Box::new(DecimalValue::from(3.14159))),
    );

    map.insert(
        "random".to_string(),
        mk_native_fn(
            "math.random".to_string(),
            Arc::new(Mutex::new(Box::new(random))),
        ),
    );

    map.insert(
        "floor".to_string(),
        mk_native_fn(
            "math.floor".to_string(),
            Arc::new(Mutex::new(Box::new(floor))),
        ),
    );

    map.insert(
        "ceil".to_string(),
        mk_native_fn(
            "math.ceil".to_string(),
            Arc::new(Mutex::new(Box::new(ceil))),
        ),
    );

    map.insert(
        "trunc".to_string(),
        mk_native_fn(
            "math.trunc".to_string(),
            Arc::new(Mutex::new(Box::new(trunc))),
        ),
    );

    map.insert(
        "round".to_string(),
        mk_native_fn(
            "math.round".to_string(),
            Arc::new(Mutex::new(Box::new(round))),
        ),
    );

    map
}
