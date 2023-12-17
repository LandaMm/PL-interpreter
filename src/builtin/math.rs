use rand::Rng;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{DecimalValue, Key, RuntimeValue, Value};

use super::{mk_native_fn, mk_runtime_value};

fn random(_args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    let mut rng = rand::thread_rng();

    let num = rng.gen::<f64>();

    mk_runtime_value(Box::new(DecimalValue::from(num)))
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

    map
}
