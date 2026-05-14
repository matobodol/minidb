use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::domain::{Column, Constraint, DataType, DomainError, Expr, Table, Value};

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
    pub fn debug(&self) -> Result<(), DomainError> {
        println!("{:#?}", self);
        Ok(())
    }
    pub fn debug_table(&self, name: &str) -> Result<(), DomainError> {
        let tbl = self.table(name)?;
        println!("{:#?}", tbl);
        Ok(())
    }
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

    pub fn table(&self, name: &str) -> Result<&Table, DomainError> {
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
    pub fn delete_columns(
        &mut self,
        table: &str,
        column: Vec<String>,
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;
        tbl.delete_columns(column)
    }
}

// ROW OPERATION
impl Database {
    pub fn insert(
        &mut self,
        table: &str,
        columns: Option<Vec<String>>,
        rows: Vec<Vec<Value>>, // multi-row
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;

        tbl.insert(columns, rows)
    }

    pub(crate) fn update_rows(
        &mut self,
        table: &str,
        assignments: Vec<(String, Value)>,
        conditions: &Expr,
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;

        tbl.update_rows(assignments, conditions)
    }

    pub(crate) fn delete_rows(
        &mut self,
        table: &str,
        conditions: &Expr,
    ) -> Result<usize, DomainError> {
        let tbl = self.table_mut(table)?;

        tbl.delete_rows(conditions)
    }
}

// Lookup API for application layer (read-only)
impl Database {
    pub fn columns(&self, table: &str) -> Result<Vec<&Column>, DomainError> {
        let tbl = self.table(table)?;
        Ok(tbl.columns().iter().collect())
    }

    pub fn selected_columns(
        &self,
        table: &str,
        columns: &[&str],
    ) -> Result<Vec<&Column>, DomainError> {
        let tbl = self.table(table)?;
        tbl.columns_selected(columns)
    }

    pub fn lookup_all(&self, table: &str) -> Result<Vec<Vec<&Value>>, DomainError> {
        let tbl = self.table(table)?;

        Ok(tbl.lookup_all())
    }

    pub fn lookup_where(
        &self,
        table: &str,
        conditions: &Expr,
    ) -> Result<Vec<Vec<&Value>>, DomainError> {
        let tbl = self.table(table)?;

        tbl.lookup_where(conditions)
    }

    pub fn lookup_columns(
        &self,
        table: &str,
        columns: &[&str],
    ) -> Result<Vec<Vec<&Value>>, DomainError> {
        let tbl = self.table(table)?;

        tbl.lookup_columns(columns)
    }

    pub fn lookup_columns_where(
        &self,
        table: &str,
        conditions: &Expr,
        columns: &[&str],
    ) -> Result<Vec<Vec<&Value>>, DomainError> {
        let tbl = self.table(table)?;

        tbl.lookup_columns_where(conditions, columns)
    }
}
