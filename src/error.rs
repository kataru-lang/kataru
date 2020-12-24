use std::fmt;

/// Error type for validating the kataru yml script.
pub struct ValidationError {
    pub message: String,
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {{ file: {}, line: {} }}", self, file!(), line!())
    }
}

#[macro_export]
macro_rules! verror {
    ($($arg:tt)*) => {{
        let res = ValidationError {
            message: format!($($arg)*)
        };
        res
    }}
}
