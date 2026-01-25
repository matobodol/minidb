use crate::database::{
    domain::{DataType, Database, Table, Value},
    engine::EngineError,
};

#[derive(Debug)]
pub struct DbEngine {
    db: Database,
}

impl DbEngine {
    pub fn new() -> Self {
        Self {
            db: Database::default(),
        }
    }

    pub fn create_table(&mut self, table_name: &str) -> Result<(), EngineError> {
        self.db
            .create_table(table_name.to_string())
            .map_err(|e| EngineError::Domain(e))
    }

    pub fn drop_table(&mut self, table: &str) -> Result<usize, EngineError> {
        self.db
            .drop_table(table)
            .map_err(|e| EngineError::Domain(e))
    }

    fn table(&self, name: &str) -> Result<&Table, EngineError> {
        self.db
            .get_table(name)
            .ok_or_else(|| EngineError::TableNotFound(name.to_string()))
    }

    fn table_mut(&mut self, name: &str) -> Result<&mut Table, EngineError> {
        self.db
            .get_table_mut(name)
            .ok_or_else(|| EngineError::TableNotFound(name.to_string()))
    }

    pub fn add_columns(
        &mut self,
        table: &str,
        columns: Vec<(String, DataType)>,
    ) -> Result<(), EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.add_column(columns).map_err(|e| EngineError::Domain(e))
    }

    pub fn insert_row(&mut self, table: &str, values: Vec<Value>) -> Result<(), EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.insert_row(values).map_err(|e| EngineError::Domain(e))
    }

    pub fn delete_row(
        &mut self,
        table: &str,
        column: &str,
        value: &Value,
    ) -> Result<usize, EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.delete_row(column, value)
            .map_err(|e| EngineError::Domain(e))
    }

    pub fn select_all(&self, table: &str) -> Result<Vec<Vec<Value>>, EngineError> {
        let tbl = self.table(table)?;

        Ok(tbl.select_all())
    }

    pub fn select_where(
        &self,
        table: &str,
        column: &str,
        value: &Value,
    ) -> Result<Vec<Vec<Value>>, EngineError> {
        let tbl = self.table(table)?;

        tbl.select_where(column, value).map_err(EngineError::Domain)
    }

    pub fn select_columns(
        &self,
        table: &str,
        columns: &[&str],
    ) -> Result<Vec<Vec<Value>>, EngineError> {
        let tbl = self.table(table)?;

        tbl.select_columns(columns).map_err(EngineError::Domain)
    }
    pub fn select_where_columns(
        &self,
        table: &str,
        where_column: &str,
        value: &Value,
        columns: &[&str],
    ) -> Result<Vec<Vec<Value>>, EngineError> {
        let tbl = self.table(table)?;

        tbl.select_where_columns(where_column, value, columns)
            .map_err(EngineError::Domain)
    }

    pub fn update_where(
        &mut self,
        table: &str,
        where_column: &str,
        where_value: &Value,
        target_column: &str,
        new_value: Value,
    ) -> Result<usize, EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.update_where(where_column, where_value, target_column, new_value)
            .map_err(EngineError::Domain)
    }
}
