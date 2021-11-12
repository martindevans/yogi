use super::*;

#[derive(Debug, PartialEq, Eq)]
pub enum Value {
    Number(Number),
    Str(YString),
}

impl Value {

    pub unsafe fn as_number_unchecked(&self) -> &Number {
        if let Value::Number(n) = self {
            n
        } else if cfg!(debug_assertions) {
            std::unreachable!()
        } else {
            std::hint::unreachable_unchecked()
        }
    }

    pub unsafe fn as_number_unchecked_mut(&mut self) -> &mut Number {
        if let Value::Number(n) = self {
            n
        } else if cfg!(debug_assertions) {
            std::unreachable!()
        } else {
            std::hint::unreachable_unchecked()
        }
    }

    pub unsafe fn as_ystring_unchecked(&self) -> &YString {
        if let Value::Str(s) = self {
            s
        } else if cfg!(debug_assertions) {
            std::unreachable!()
        } else {
            std::hint::unreachable_unchecked()
        }
    }

    pub unsafe fn as_ystring_unchecked_mut(&mut self) -> &mut YString {
        if let Value::Str(s) = self {
            s
        } else if cfg!(debug_assertions) {
            std::unreachable!()
        } else {
            std::hint::unreachable_unchecked()
        }
    }

    pub fn le(&self, other: &Self, buffer: &mut String) -> bool {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l <= r,
            (Value::Number(l), Value::Str(r)) => l.stringify(buffer) <= *r,
            (Value::Str(l), Value::Number(r)) => *l <= r.stringify(buffer),
            (Value::Str(l), Value::Str(r)) => l <= r,
        }
    }

    pub fn lt(&self, other: &Self, buffer: &mut String) -> bool {
        match (self, other) {
            (Value::Number(l), Value::Number(r)) => l < r,
            (Value::Number(l), Value::Str(r)) => l.stringify(buffer) < *r,
            (Value::Str(l), Value::Number(r)) => *l < r.stringify(buffer),
            (Value::Str(l), Value::Str(r)) => l < r,
        }
    }

    pub fn add_assign(&mut self, other: &Self, buffer: &mut String) {
        match (&mut *self, other) {
            (Value::Number(l), &Value::Number(r)) => { *l += r; },
            (Value::Number(l), Value::Str(r)) => {
                let mut l: YString = l.stringify(buffer);
                l += r;
                *self = Value::Str(l);
            },
            (Value::Str(l), Value::Number(r)) => {
                *l += &r.stringify(buffer);
            },
            (Value::Str(l), Value::Str(r)) => {
                *l += r;
            },
        }
    }

    pub fn sub_assign(&mut self, other: &Self, buffer: &mut String) {
        match (&mut *self, other) {
            (Value::Number(l), &Value::Number(r)) => { *l -= r; },
            (Value::Number(l), Value::Str(r)) => {
                let mut l: YString = l.stringify(buffer);
                l -= r;
                *self = Value::Str(l);
            },
            (Value::Str(l), Value::Number(r)) => {
                *l -= &r.stringify(buffer);
            },
            (Value::Str(l), Value::Str(r)) => {
                *l -= r;
            },
        }
    }

    pub fn pre_inc(&mut self) {
        match self {
            Value::Number(n) => n.pre_inc(),
            Value::Str(s) => s.data.insert(0, ' '),
        }
    }

    pub fn pre_dec(&mut self) -> ValueResult<()> {
        match self {
            Value::Number(n) => {
                n.pre_dec();
                Ok(())
            },
            Value::Str(s) => {
                s.pre_dec()
            },
        }
    }

    pub fn as_bool(&self) -> bool {
        match self {
            Value::Number(n) => n.as_bool(),
            Value::Str(_) => false,
        }
    }

    /// Returns `true` if the value is [`Number`].
    ///
    /// [`Number`]: Value::Number
    pub const fn is_number(&self) -> bool {
        matches!(self, Self::Number(..))
    }

    /// Returns `true` if the value is [`Str`].
    ///
    /// [`Str`]: Value::Str
    pub const fn is_str(&self) -> bool {
        matches!(self, Self::Str(..))
    }

    pub const fn as_number(&self) -> Option<Number> {
        if let &Self::Number(n) = self {
            Some(n)
        } else {
            None
        }
    }

    pub fn as_number_mut(&mut self) -> Option<&mut Number> {
        if let Self::Number(n) = self {
            Some(n)
        } else {
            None
        }
    }

    pub const fn as_ystring(&self) -> Option<&YString> {
        if let Self::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }

    pub fn as_ystring_mut(&mut self) -> Option<&mut YString> {
        if let Self::Str(s) = self {
            Some(s)
        } else {
            None
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Number(n) => Value::Number(n.clone()),
            Value::Str(s) => Value::Str(s.clone()),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (Value::Str(l), Value::Str(r)) => {
                l.clone_from(r);
            },
            (l, r) => {
                *l = r.clone();
            },
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        match self {
            Value::Number(n) => write!(f, "{}", n),
            Value::Str(s) => write!(f, "{}", s),
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::Number(Number::ZERO)
    }
}

impl Not for Value {
    type Output = Number;

    fn not(self) -> Self::Output {
        !&self
    }
}

impl Not for &'_ Value {
    type Output = Number;

    fn not(self) -> Self::Output {
        if let &Value::Number(n) = self {
            !n
        } else {
            Number::ZERO
        }
    }
}

impl From<YString> for Value {
    fn from(s: YString) -> Self {
        Value::Str(s)
    }
}

impl From<Number> for Value {
    fn from(n: Number) -> Self {
        Value::Number(n)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExpectedTy {
    Number,
    Str,
}

impl Display for ExpectedTy {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(match self {
            ExpectedTy::Number => "number",
            ExpectedTy::Str => "string",
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum WrongArgType {
    Left,
    Right,
    Both,
    Only,
}

impl Display for WrongArgType {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        f.write_str(match self {
            WrongArgType::Left => "left arg",
            WrongArgType::Right => "right arg",
            WrongArgType::Both => "both args",
            WrongArgType::Only => "the arg",
        })
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum RuntimeErr {
    #[error("expected a {0} in {1}")]
    Expected(ExpectedTy, WrongArgType),
    #[error("string was empty")]
    EmptyStr,
    #[error("division by zero")]
    DivZero,
    #[error("mod by zero")]
    ModZero,
}

pub type ValueResult<T> = Result<T, RuntimeErr>;