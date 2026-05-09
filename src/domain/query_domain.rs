use crate::domain::{Cmp, Value};

#[derive(Debug, Clone)]
pub struct ResolvedCondition {
    pub index: usize,
    pub cmp: Cmp,
    pub value: Option<Value>, // penting
}

impl ResolvedCondition {
    pub fn eq(index: usize, value: Option<Value>) -> Self {
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
    pub value: Option<Value>,
}
impl Condition {
    pub fn new(column: String, cmp: Cmp, value: Option<Value>) -> Self {
        Self { column, cmp, value }
    }
}
