macro_rules! bail {
    ($e:expr) => {
        return Err($e.into())
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}

pub(crate) use bail;
