use crate::database::domain::{Cmp, Value};

#[derive(Debug, Clone)]
pub struct Row {
    values: Vec<Value>,
}

impl Row {
    pub(super) fn len(&self) -> usize {
        self.values.len()
    }
    pub(super) fn backfill_push(&mut self, value: Value) {
        self.values.push(value);
    }
    pub fn new(values: Vec<Value>) -> Self {
        Self { values }
    }

    pub(super) fn delete_cell(&mut self, index: usize) {
        self.values.remove(index);
    }

    pub fn get_values(&self) -> &Vec<Value> {
        &self.values
    }

    pub fn get(&self, index: usize) -> &Value {
        // index wajib hasil resolve oleh schema
        &self.values[index]
    }

    pub(crate) fn set_value(&mut self, index: usize, value: Value) {
        if let Some(slot) = self.values.get_mut(index) {
            *slot = value;
        }
    }
}

impl Row {
    /// Mengecek kecocokan value pada index tertentu
    pub(crate) fn value_is_match(&self, index: usize, cmp: &Cmp, value: &Value) -> bool {
        let Some(selected) = self.values.get(index) else {
            return false;
        };

        selected.compare(cmp, value)
    }
}
