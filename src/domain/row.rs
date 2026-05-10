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

    pub(super) fn remove_at(&mut self, index: usize) {
        self.values.remove(index);
    }

    pub(super) fn values(&self) -> &Vec<Value> {
        &self.values
    }
}
