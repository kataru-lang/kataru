use super::Value;
use std::fmt;
use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, Div, DivAssign, Mul, MulAssign, Neg,
    Not, Sub, SubAssign,
};

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::String(v) => write!(f, "{}", v),
            Self::Number(v) => write!(f, "{}", v),
            Self::Bool(v) => write!(f, "{}", v),
        }
    }
}

impl AddAssign<Self> for Value {
    fn add_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Self::Number(n1 + n2),
            (Value::String(s1), Value::String(ref s2)) => Self::String(format!("{}{}", s1, s2)),
            _ => return,
        };
    }
}

impl Add<Self> for Value {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result += rhs;
        result
    }
}

impl SubAssign<Self> for Value {
    fn sub_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Self::Number(n1 - n2),
            _ => return,
        };
    }
}

impl Sub<Self> for Value {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result -= rhs;
        result
    }
}

impl MulAssign<Self> for Value {
    fn mul_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => Self::Number(n1 * n2),
            _ => return,
        }
    }
}

impl MulAssign<f64> for Value {
    fn mul_assign(&mut self, rhs: f64) {
        if let Value::Number(n1) = &self {
            *self = Self::Number(n1 * rhs)
        }
    }
}

impl Mul<Self> for Value {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result *= rhs;
        result
    }
}

impl DivAssign<Self> for Value {
    fn div_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Value::Number(n1), Value::Number(n2)) => {
                if n2 == 0. {
                    Self::Number(0.)
                } else {
                    Self::Number(n1 / n2)
                }
            }
            _ => return,
        }
    }
}

impl DivAssign<f64> for Value {
    fn div_assign(&mut self, rhs: f64) {
        if let Value::Number(n1) = &self {
            *self = {
                if rhs == 0. {
                    Self::Number(0.)
                } else {
                    Self::Number(n1 / rhs)
                }
            }
        }
    }
}

impl Div<Self> for Value {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result /= rhs;
        result
    }
}

impl BitAndAssign<Self> for Value {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Value::Bool(b1), Value::Bool(b2)) => Self::Bool(b1 & b2),
            _ => return,
        }
    }
}

impl BitAnd<Self> for Value {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result &= rhs;
        result
    }
}

impl BitOrAssign<Self> for Value {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = match (&self, rhs) {
            (Value::Bool(b1), Value::Bool(b2)) => Self::Bool(b1 | b2),
            _ => return,
        }
    }
}

impl BitOr<Self> for Value {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        let mut result = self.clone();
        result |= rhs;
        result
    }
}

impl Not for Value {
    type Output = Value;
    fn not(self) -> Self::Output {
        match self {
            Value::Bool(b) => Self::Bool(!b),
            _ => self,
        }
    }
}

impl Neg for Value {
    type Output = Value;
    fn neg(self) -> Self::Output {
        match self {
            Value::Number(n) => Self::Number(-n),
            _ => self,
        }
    }
}
