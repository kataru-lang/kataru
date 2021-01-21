use std::fmt;

/// Error type for validating the kataru yml script.
pub struct ParseError {
    pub message: String,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Debug for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {{ file: {}, line: {} }}", self, file!(), line!())
    }
}

#[macro_export]
macro_rules! perror {
    ($($arg:tt)*) => {{
        let res = ParseError {
            message: format!($($arg)*)
        };
        res
    }}
}
