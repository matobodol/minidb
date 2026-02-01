use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{Column, Condition, Constraint, DataType, DomainError, Table, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    tables: HashMap<String, Table>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            tables: HashMap::new(),
        }
    }
}

impl Database {
    pub fn create_table(&mut self, name: &str) -> Result<(), DomainError> {
        if self.tables.contains_key(name) {
            return Err(DomainError::DuplicateTableName);
        }
        self.tables.insert(name.to_string(), Table::new());

        Ok(())
    }

    pub fn drop_table(&mut self, name: &str) -> Result<usize, DomainError> {
        let before = self.tables.len();

        self.tables
            .remove(name)
            .map(|_| before - self.tables.len())
            .ok_or(DomainError::TableNotFound(name.to_string()))
    }

    pub fn list_tables(&self) -> Vec<String> {
        self.tables.keys().cloned().collect()
    }

    pub fn describe_table(&self, name: &str) -> Result<Vec<Column>, DomainError> {
        let table = self.table(name)?;

        Ok(table.columns().to_vec())
    }

    fn table(&self, name: &str) -> Result<&Table, DomainError> {
        self.tables
            .get(name)
            .ok_or(DomainError::TableNotFound(name.to_string()))
    }

    fn table_mut(&mut self, name: &str) -> Result<&mut Table, DomainError> {
        self.tables
            .get_mut(name)
            .ok_or(DomainError::TableNotFound(name.to_string()))
    }
}

// COLUMN OPERATION
impl Database {
    pub fn add_columns(
        &mut self,
        table: &str,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        let tbl = self.table_mut(table)?;
        tbl.add_column(columns)
    }
    pub fn delete_column(
        &mut self,
        table: &str,
        column: Vec<String>,
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;
        tbl.delete_column(column)
    }
}

// ROW OPERATION
impl Database {
    pub fn insert_row(&mut self, table: &str, values: &[(&str, Value)]) -> Result<(), DomainError> {
        let tbl = self.table_mut(table)?;

        tbl.insert_row(values)
    }

    pub(crate) fn update_where(
        &mut self,
        table: &str,
        conditions: &[Condition],
        assignments: &[(String, Value)],
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;
        tbl.update_where(conditions, assignments)
    }

    pub(crate) fn delete_where(
        &mut self,
        table: &str,
        conditions: &[Condition],
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;
        tbl.delete_row(conditions)
    }
}

// Lookup API for application layer (read-only)
impl Database {
    pub fn select_all(&self, table: &str) -> Result<Vec<Vec<String>>, DomainError> {
        let tbl = self.table(table)?;

        Ok(tbl.select_all())
    }

    pub fn select_where(
        &self,
        table: &str,
        condition: Condition,
    ) -> Result<Vec<Vec<String>>, DomainError> {
        let tbl = self.table(table)?;

        tbl.select_where(condition)
    }

    pub fn select_columns(
        &self,
        table: &str,
        columns: &[&str],
    ) -> Result<Vec<Vec<String>>, DomainError> {
        let tbl = self.table(table)?;

        tbl.select_columns(columns)
    }

    pub fn select_where_columns(
        &self,
        table: &str,
        condition: Condition,
        columns: &[&str],
    ) -> Result<Vec<Vec<String>>, DomainError> {
        let tbl = self.table(table)?;

        tbl.select_where_columns(condition, columns)
    }
}
