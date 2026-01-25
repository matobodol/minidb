use crate::database::domain::{DataType, DomainError, Row, Schema, Value};

#[derive(Debug, Clone)]
pub struct Table {
    schema: Schema,
    rows: Vec<Row>,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
            schema: Schema::default(),
            rows: Vec::new(),
        }
    }

    pub(crate) fn add_column(
        &mut self,
        columns: Vec<(String, DataType)>,
    ) -> Result<(), DomainError> {
        self.schema.add_column(columns)?;

        for row in &mut self.rows {
            if row.len() < self.schema.len() {
                row.backfill_push(Value::Null);
            }
        }
        Ok(())
    }

    fn index_column(&self, column: &str) -> Result<usize, DomainError> {
        self.schema
            .get_index(column)
            .ok_or(DomainError::ColumnNotFound)
    }

    pub(crate) fn insert_row(&mut self, values: Vec<Value>) -> Result<(), DomainError> {
        self.schema.validate_row(&values)?;

        let row = Row::new(values);
        self.rows.push(row);

        Ok(())
    }

    pub(crate) fn delete_row(&mut self, column: &str, value: &Value) -> Result<usize, DomainError> {
        let index = self.index_column(column)?;

        let before = self.rows.len();
        self.rows.retain(|row| !row.matched(index, value));

        let after = self.rows.len();
        if before == after {
            return Err(DomainError::ValueNotFound {
                miss_value: value.clone(),
                in_the_column: column.to_string(),
                reason: format!("The value is not listed in the selected column.",),
            });
        };

        Ok(before - after)
    }

    pub(crate) fn select_all(&self) -> Vec<Vec<Value>> {
        self.rows
            .iter()
            .map(|row| row.get_value().clone())
            .collect()
    }

    pub(crate) fn select_where(
        &self,
        column: &str,
        value: &Value,
    ) -> Result<Vec<Vec<Value>>, DomainError> {
        let index = self.index_column(column)?;

        let result = self
            .rows
            .iter()
            .filter(|row| row.matched(index, value))
            .map(|row| row.get_value().clone())
            .collect();

        Ok(result)
    }

    pub(crate) fn select_columns(&self, columns: &[&str]) -> Result<Vec<Vec<Value>>, DomainError> {
        // 1. resolve index kolom (sekali di awal)
        let indices: Vec<usize> = columns
            .iter()
            .map(|name| self.index_column(name))
            .collect::<Result<_, _>>()?;

        // 2. ambil value sesuai index
        let result = self
            .rows
            .iter()
            .map(|row| {
                indices
                    .iter()
                    .map(|&i| row.get_value()[i].clone())
                    .collect::<Vec<Value>>()
            })
            .collect();

        Ok(result)
    }
    pub(crate) fn select_where_columns(
        &self,
        column: &str,
        value: &Value,
        columns: &[&str],
    ) -> Result<Vec<Vec<Value>>, DomainError> {
        // 1. resolve index where
        let where_index = self.index_column(column)?;

        // 2. resolve index projection (sekali di awal)
        let projection_indices: Vec<usize> = columns
            .iter()
            .map(|name| self.index_column(name))
            .collect::<Result<_, _>>()?;

        // 3. filter + project
        let result = self
            .rows
            .iter()
            .filter(|row| row.matched(where_index, value))
            .map(|row| {
                projection_indices
                    .iter()
                    .map(|&i| row.get_value()[i].clone())
                    .collect::<Vec<Value>>()
            })
            .collect();

        Ok(result)
    }

    pub(crate) fn update_where(
        &mut self,
        where_column: &str,
        where_value: &Value,
        target_column: &str,
        new_value: Value,
    ) -> Result<usize, DomainError> {
        // 1. resolve index
        let where_index = self.index_column(where_column)?;
        let target_index = self.index_column(target_column)?;

        // 2. validasi tipe data (sekali)
        let column = &self.schema.columns()[target_index];
        if !new_value.matches(column.data_type()) {
            return Err(DomainError::TypeMismatch {
                column_index: target_index,
                expected: column.data_type().clone(),
                found: new_value.clone(),
            });
        }

        // 3. update rows
        let mut updated = 0;

        for row in self.rows.iter_mut() {
            if row.matched(where_index, where_value) {
                row.set_value(target_index, new_value.clone());
                updated += 1;
            }
        }

        if updated == 0 {
            return Err(DomainError::ValueNotFound {
                miss_value: where_value.clone(),
                in_the_column: where_column.to_string(),
                reason: "No rows matched update 'where' condition".into(),
            });
        }

        Ok(updated)
    }
}
