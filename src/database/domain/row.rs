use crate::database::domain::Value;

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

    pub fn get_value(&self) -> &Vec<Value> {
        &self.values
    }

    pub(crate) fn set_value(&mut self, index: usize, value: Value) {
        if let Some(slot) = self.values.get_mut(index) {
            *slot = value;
        }
    }

    pub(crate) fn matched(&self, index: usize, value: &Value) -> bool {
        let Some(selected) = self.values.get(index) else {
            return false;
        };
        match (selected, value) {
            (Value::Null, Value::Null) => true,
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Enum { value: a }, Value::Enum { value: b }) => a == b,
            _ => false,
        }
    }
}
