use crate::database::{
    domain::Value,
    engine::{EngineError, MiniDBEngine},
};

// Lookup API for application layer (read-only)
impl MiniDBEngine {
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
}
