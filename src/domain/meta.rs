use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TableMeta {
    increments: HashMap<String, i64>,
}
impl TableMeta {
    pub(super) fn next_increment(&mut self, column: &str) -> i64 {
        let entry = self.increments.entry(column.to_string()).or_insert(1);

        let current = *entry;

        *entry += 1;

        current
    }

    pub(super) fn sync_increment(&mut self, column: &str, value: i64) {
        let entry = self.increments.entry(column.to_string()).or_insert(1);

        if value >= *entry {
            *entry = value + 1;
        }
    }

    pub(super) fn remove_increment(&mut self, column: &str) {
        self.increments.remove(column);
    }
}
