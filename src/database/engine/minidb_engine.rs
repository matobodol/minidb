use crate::database::{
    domain::{Condition, DataType, Database, Table, Value},
    engine::EngineError,
};

#[derive(Debug)]
pub struct MiniDBEngine {
    db: Database,
}

// DATABASE OPERATION
impl MiniDBEngine {
    pub fn new() -> Self {
        Self {
            db: Database::default(),
        }
    }

    pub(super) fn table(&self, name: &str) -> Result<&Table, EngineError> {
        self.db
            .get_table(name)
            .ok_or_else(|| EngineError::TableNotFound(name.to_string()))
    }

    fn table_mut(&mut self, name: &str) -> Result<&mut Table, EngineError> {
        self.db
            .get_table_mut(name)
            .ok_or_else(|| EngineError::TableNotFound(name.to_string()))
    }
}

// TABLE OPERATION
impl MiniDBEngine {
    pub fn create_table(&mut self, table_name: &str) -> Result<(), EngineError> {
        self.db
            .create_table(table_name.to_string())
            .map_err(EngineError::Domain)
    }

    pub fn drop_table(&mut self, table: &str) -> Result<usize, EngineError> {
        self.db.drop_table(table).map_err(EngineError::Domain)
    }
}

// COLUMN OPERATION
impl MiniDBEngine {
    pub fn add_columns(
        &mut self,
        table: &str,
        columns: Vec<(&str, DataType)>,
    ) -> Result<(), EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.add_column(columns).map_err(|e| EngineError::Domain(e))
    }
    pub fn delete_column(&mut self, table: &str, column: &str) -> Result<usize, EngineError> {
        let tbl = self.table_mut(table)?;
        tbl.delete_column(column)
            .map_err(|e| EngineError::Domain(e))
    }
}

// ROW OPERATION
impl MiniDBEngine {
    pub fn insert_row(&mut self, table: &str, values: &[(&str, Value)]) -> Result<(), EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.insert_row(values).map_err(|e| EngineError::Domain(e))
    }

    pub fn update_row_where(
        &mut self,
        table: &str,
        conditions: &[Condition],
        target: (&str, Value),
    ) -> Result<usize, EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.update_row_where(conditions, target)
            .map_err(|e| EngineError::Domain(e))
    }

    pub fn delete_row_where(
        &mut self,
        table: &str,
        conditions: &[Condition],
    ) -> Result<usize, EngineError> {
        let tbl = self.table_mut(table)?;

        tbl.delete_row_where(conditions)
            .map_err(|e| EngineError::Domain(e))
    }
}
