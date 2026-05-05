use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Int,
    Str,
    Float,
    // Date,
    Enum { variants: HashSet<String> },
}
impl DataType {
    pub fn matches_type(&self, value: &Value) -> bool {
        self.coerce_value(value.clone()).is_ok()
    }
    pub fn enum_of(values: Vec<String>) -> Self {
        if values.is_empty() {
            panic!("Enum must not be empty");
        }

        if values.iter().any(|v| v.trim().is_empty()) {
            panic!("Enum variants cannot be empty");
        }

        let set: HashSet<_> = values.into_iter().collect();

        Self::Enum { variants: set }
    }

    pub fn coerce_value(&self, value: Value) -> Result<Value, DomainError> {
        match (self, value) {
            // INT
            (DataType::Int, Value::Int(v)) => Ok(Value::Int(v)),

            (DataType::Int, Value::Str(s)) => s
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| DomainError::TypeMismatch),

            // FLOAT
            (DataType::Float, Value::Float(v)) => Ok(Value::Float(v)),

            (DataType::Float, Value::Str(s)) => s
                .parse::<f64>()
                .map(Value::Float)
                .map_err(|_| DomainError::TypeMismatch),

            // STRING
            (DataType::Str, Value::Str(s)) => Ok(Value::Str(s)),

            // ENUM
            (DataType::Enum { variants }, Value::Str(s)) => {
                if variants.contains(&s) {
                    Ok(Value::Enum { value: s })
                } else {
                    Err(DomainError::InvalidDefaultType)
                }
            }

            (DataType::Enum { variants }, Value::Enum { value }) => {
                if variants.contains(&value) {
                    Ok(Value::Enum { value })
                } else {
                    Err(DomainError::InvalidDefaultType)
                }
            }

            // NULL
            (_, Value::Null) => Ok(Value::Null),

            _ => Err(DomainError::TypeMismatch),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Value {
    Int(i64),
    Str(String),
    Float(f64),
    Enum { value: String },

    // Date(chrono::NaiveDate),
    Null,
}

impl Value {
    pub fn compare(&self, op: &Cmp, other: &Value) -> bool {
        if matches!(self, Value::Null) || matches!(other, Value::Null) {
            return false;
        }

        match op {
            Cmp::Eq => self.eq(other),
            Cmp::Ne => !self.eq(other),
            Cmp::Lt => self.lt(other),
            Cmp::Gt => self.gt(other),
            Cmp::Lte => self.le(other),
            Cmp::Gte => self.ge(other),
            Cmp::IsNull => matches!(other, Value::Null),
            Cmp::IsNotNull => !matches!(other, Value::Null),
        }
    }

    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Enum { value: a }, Value::Enum { value: b }) => a == b,
            _ => false,
        }
    }

    fn lt(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a < b,
            (Value::Float(a), Value::Float(b)) => a < b,
            _ => false,
        }
    }

    fn gt(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a > b,
            (Value::Float(a), Value::Float(b)) => a > b,
            _ => false,
        }
    }

    fn le(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a <= b,
            (Value::Float(a), Value::Float(b)) => a <= b,
            _ => false,
        }
    }

    fn ge(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a >= b,
            (Value::Float(a), Value::Float(b)) => a >= b,
            _ => false,
        }
    }
}

impl Value {
    pub(crate) fn to_display_str(&self) -> String {
        match self {
            Value::Int(v) => v.to_string(),
            Value::Str(v) => v.clone(),
            Value::Float(v) => v.to_string(),
            Value::Enum { value } => value.clone(),
            Value::Null => "-".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cmp {
    Eq,
    Ne,
    Lt,
    Gt,
    Lte,
    Gte,
    IsNull,
    IsNotNull,
}
