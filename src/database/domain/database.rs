use std::collections::HashMap;

use crate::database::domain::{DomainError, Table};

#[derive(Default, Debug, Clone)]
pub struct Database {
    tables: HashMap<String, Table>,
}

impl Database {
    pub(crate) fn create_table(&mut self, name: String) -> Result<(), DomainError> {
        if self.contains_table(&name) {
            return Err(DomainError::DuplicateTableName);
        }
        self.tables.insert(name, Table::new());

        Ok(())
    }

    pub(crate) fn drop_table(&mut self, name: &str) -> Result<usize, DomainError> {
        let before = self.tables.len();

        self.tables
            .remove(name)
            .map(|_| before - self.tables.len())
            .ok_or(DomainError::TableNotFound(name.to_string()))
    }

    pub(crate) fn get_table(&self, name: &str) -> Option<&Table> {
        self.tables.get(name)
    }

    pub(crate) fn get_table_mut(&mut self, name: &str) -> Option<&mut Table> {
        self.tables.get_mut(name)
    }

    pub(crate) fn contains_table(&self, name: &str) -> bool {
        self.tables.contains_key(name)
    }
}
