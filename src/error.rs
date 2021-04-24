use std::fmt;
use std::hash::Hash;

/// Error type for validating the kataru yml script.
#[derive(PartialEq)]
pub enum Error {
    Generic(String),
    Pest(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Generic(message) => write!(f, "{}", message),
            Self::Pest(message) => write!(f, "{}", message),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}, {{ file: {}, line: {} }}", self, file!(), line!())
    }
}

impl<R> From<pest::error::Error<R>> for Error
where
    R: Ord + Copy + fmt::Debug + Hash,
{
    fn from(pest_error: pest::error::Error<R>) -> Self {
        Self::Pest(pest_error.to_string())
    }
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {{
        let res = Error::Generic(format!($($arg)*));
        res
    }}
}
