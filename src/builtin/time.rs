use std::{
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{IntegerValue, RuntimeValue};

use super::mk_runtime_value;

pub fn native_get_time(
    _args: Vec<Arc<Mutex<Box<dyn RuntimeValue>>>>,
) -> Arc<Mutex<Box<dyn RuntimeValue>>> {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
    mk_runtime_value(Box::new(IntegerValue::from(
        since_the_epoch.as_millis() as isize
    )))
}
