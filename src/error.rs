use std::fmt;

/// Error type for validating the kataru yml script.
pub struct Error {
    pub message: String,
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {{ file: {}, line: {} }}", self, file!(), line!())
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let res = Error {
            message: format!($($arg)*)
        };
        res
    }}
}
