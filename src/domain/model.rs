use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::domain::DomainError;

use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DataType {
    Int,
    Str,
    Float,
    Enum { variants: HashSet<String> },
}

impl DataType {
    pub fn matches_type(&self, value: &Value) -> bool {
        match (self, value) {
            (DataType::Int, Value::Int(_)) => true,
            (DataType::Str, Value::Str(_)) => true,
            (DataType::Float, Value::Float(_)) => true,

            (DataType::Enum { variants }, Value::Enum { value }) => variants.contains(value),

            (_, Value::Null) => true,

            _ => false,
        }
    }

    pub(crate) fn enum_of(values: Vec<String>) -> Result<Self, DomainError> {
        if values.is_empty() {
            return Err(DomainError::EmptyEnumVariant);
        }

        if values.iter().any(|v| v.trim().is_empty()) {
            return Err(DomainError::EmptyEnumVariant);
        }

        Ok(Self::Enum {
            variants: values.into_iter().collect(),
        })
    }

    pub(crate) fn coerce_value(&self, value: Value) -> Result<Value, DomainError> {
        match (self, value) {
            // INT
            (DataType::Int, Value::Int(v)) => Ok(Value::Int(v)),

            (DataType::Int, Value::Str(s)) => s
                .parse::<i64>()
                .map(Value::Int)
                .map_err(|_| DomainError::TypeMismatch),

            // FLOAT
            (DataType::Float, Value::Float(v)) => Ok(Value::Float(v)),

            (DataType::Float, Value::Int(v)) => Ok(Value::Float(v as f64)),

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
                    Err(DomainError::InvalidEnumValue)
                }
            }

            (DataType::Enum { variants }, Value::Enum { value }) => {
                if variants.contains(&value) {
                    Ok(Value::Enum { value })
                } else {
                    Err(DomainError::InvalidEnumValue)
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
    Null,
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => write!(f, "{v}"),
            Value::Str(v) => write!(f, "{v}"),
            Value::Enum { value } => write!(f, "{value}"),
            Value::Null => write!(f, ""),
        }
    }
}
