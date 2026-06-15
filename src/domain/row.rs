// domain/row.rs
use serde::{Deserialize, Serialize};

use crate::domain::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    pub(super) fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub(super) fn append(&mut self, value: Value) {
        self.values.push(value);
    }

    pub(super) fn reserve(&mut self, additional: usize) {
        self.values.reserve(additional);
    }

    pub(super) fn remove_at(&mut self, index: usize) {
        self.values.remove(index);
    }

    pub(super) fn values(&self) -> &[Value] {
        &self.values
    }
}
