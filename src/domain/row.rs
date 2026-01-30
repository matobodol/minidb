use serde::{Deserialize, Serialize};

use crate::domain::{ResolvedCondition, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub(super) fn append(&mut self, value: Value) {
        self.values.push(value);
    }

    pub(super) fn remove_at(&mut self, index: usize) {
        self.values.remove(index);
    }

    pub fn values(&self) -> &Vec<Value> {
        &self.values
    }

    pub(crate) fn replace(&mut self, index: usize, value: Value) {
        if let Some(slot) = self.values.get_mut(index) {
            *slot = value;
        }
    }
}

impl Row {
    /// Mengecek kecocokan value pada index tertentu
    pub(crate) fn value_is_match(&self, cond: &ResolvedCondition) -> bool {
        let Some(selected) = self.values.get(cond.index) else {
            return false;
        };

        selected.compare(&cond.cmp, &cond.value)
    }
}
