use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    cast_value, DecimalValue, IntegerValue, Key, NullValue, RuntimeValue, Value, ValueType,
};

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
    let value = arg
        .lock()
        .expect("math.floor: failed to get first argument");

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
    let value = arg.lock().expect("math.ceil: failed to get first argument");

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
    let value = arg
        .lock()
        .expect("math.trunc: failed to get first argument");

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
    let value = arg
        .lock()
        .expect("math.round: failed to get first argument");

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

fn pow(args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.len() != 2 {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let target_val = args
        .get(0)
        .unwrap()
        .lock()
        .expect("math.pow: failed to get first argument");
    let factor_val = args
        .get(1)
        .unwrap()
        .lock()
        .expect("math.pow: failed to get second argument");

    if target_val.kind() != ValueType::Decimal && target_val.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if factor_val.kind() != ValueType::Decimal && factor_val.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if target_val.kind() == ValueType::Decimal || factor_val.kind() == ValueType::Decimal {
        let target = if target_val.kind() == ValueType::Decimal {
            cast_value::<DecimalValue>(&target_val).unwrap().value()
        } else {
            cast_value::<IntegerValue>(&target_val).unwrap().value() as f64
        };

        let factor = if factor_val.kind() == ValueType::Decimal {
            cast_value::<DecimalValue>(&factor_val).unwrap().value()
        } else {
            cast_value::<IntegerValue>(&factor_val).unwrap().value() as f64
        };

        return mk_runtime_value(Box::new(DecimalValue::from(target.powf(factor))));
    }

    let target = cast_value::<IntegerValue>(&target_val).unwrap().value();
    let factor = cast_value::<IntegerValue>(&factor_val).unwrap().value();

    return mk_runtime_value(Box::new(IntegerValue::from(target.pow(factor as u32))));
}

fn sqrt(args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    if args.is_empty() {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    let arg = args.get(0).unwrap();
    let value = arg.lock().expect("math.sqrt: failed to get first argument");

    if value.kind() != ValueType::Decimal && value.kind() != ValueType::Integer {
        return mk_runtime_value(Box::new(NullValue::default()));
    }

    if value.kind() == ValueType::Decimal {
        let decimal = cast_value::<DecimalValue>(&value).unwrap();
        return mk_runtime_value(Box::new(DecimalValue::from(decimal.value().sqrt())));
    }

    if value.kind() == ValueType::Integer {
        let integer = cast_value::<IntegerValue>(&value).unwrap();
        return mk_runtime_value(Box::new(DecimalValue::from(
            (integer.value() as f64).sqrt(),
        )));
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

    map.insert(
        "pow".to_string(),
        mk_native_fn("math.pow".to_string(), Arc::new(Mutex::new(Box::new(pow)))),
    );

    map.insert(
        "sqrt".to_string(),
        mk_native_fn(
            "math.sqrt".to_string(),
            Arc::new(Mutex::new(Box::new(sqrt))),
        ),
    );

    map
}
