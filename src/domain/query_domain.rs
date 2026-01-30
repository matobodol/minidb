use crate::domain::{Cmp, Value};

#[derive(Debug, Clone)]
pub struct ResolvedCondition {
    pub index: usize,
    pub cmp: Cmp,
    pub value: Value,
}
impl ResolvedCondition {
    pub fn eq(index: usize, value: Value) -> Self {
        Self {
            index,
            cmp: Cmp::Eq,
            value,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Condition {
    pub column: String,
    pub cmp: Cmp,
    pub value: Value,
}
impl Condition {
    pub fn eq(column: &str, value: Value) -> Self {
        Self {
            column: column.to_string(),
            cmp: Cmp::Eq,
            value,
        }
    }

    pub fn lt(column: &str, value: Value) -> Self {
        Self {
            column: column.to_string(),
            cmp: Cmp::Lt,
            value,
        }
    }

    pub fn gt(column: &str, value: Value) -> Self {
        Self {
            column: column.to_string(),
            cmp: Cmp::Gt,
            value,
        }
    }
}
