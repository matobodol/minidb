use crate::database::domain::{
    Cmp, Condition, DataType, DomainError, ResolvedCondition, Row, Schema, Value,
};

#[derive(Debug, Clone)]
pub(crate) struct Table {
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

    // filter: bekerja bersama (row.value_is_match())
    // cari index baris. bukan index values
    pub(crate) fn find_rows_by_resolved_conditions_desc(
        &self,
        conditions: &[ResolvedCondition],
    ) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.row_matches_all(row, conditions))
            .map(|(i, _)| i)
            .rev()
            .collect()
    }

    fn row_matches_all(&self, row: &Row, conditions: &[ResolvedCondition]) -> bool {
        conditions
            .iter()
            .all(|cond| row.value_is_match(cond.index, &cond.cmp, &cond.value))
    }
}

// COLUMN OPERATION
impl Table {
    pub(crate) fn add_column(&mut self, columns: Vec<(&str, DataType)>) -> Result<(), DomainError> {
        self.schema.add_column(columns)?;

        for row in &mut self.rows {
            if row.len() < self.schema.len() {
                row.backfill_push(Value::Null);
            }
        }
        Ok(())
    }

    pub(crate) fn delete_column(&mut self, column: &str) -> Result<usize, DomainError> {
        let index = self.schema.validate_resolve_column(column)?;

        let afected = self.schema.delete(index)?;
        self.rows
            .iter_mut()
            .for_each(|values| values.delete_cell(index));

        Ok(afected)
    }
}

// ROW OPERATION FINAL
impl Table {
    pub(crate) fn insert_row(&mut self, values: &[(&str, Value)]) -> Result<(), DomainError> {
        // buffer awal: None (belum terisi)
        let mut buffer: Vec<Option<Value>> = vec![None; self.schema.len()];

        // isi dari input
        for (name, value) in values {
            let index = self.schema.validate_resolve_column(name)?;

            // cegah duplicat insert
            if buffer[index].is_some() {
                return Err(DomainError::DuplicateColumnName);
            }

            // validasi tipe
            self.schema.validate_update(index, value)?;
            buffer[index] = Some(value.clone());
        }

        // finalisasi row (default → nullable → error)
        let row_values: Vec<Value> = buffer
            .into_iter()
            .enumerate()
            .map(|(i, slot)| {
                let column = &self.schema.columns()[i];

                match slot {
                    Some(v) => Ok(v),
                    None => {
                        if let Some(default) = column.default_value() {
                            Ok(default.clone())
                        } else if column.is_nullable() {
                            Ok(Value::Null)
                        } else {
                            Err(DomainError::NotAllowedNull)
                        }
                    }
                }
            })
            .collect::<Result<_, _>>()?;

        self.schema.validate_row(&row_values)?;
        self.rows.push(Row::new(row_values));
        Ok(())
    }

    pub(crate) fn update_row_where(
        &mut self,
        conditions: &[Condition],
        target: (&str, Value),
    ) -> Result<usize, DomainError> {
        let (target_col, target_val) = target;
        let target_index = self.schema.validate_resolve_column(target_col)?;

        self.schema.validate_update(target_index, &target_val)?;

        let resolved = self.schema.resolve_conditions(conditions)?;
        let indices = self.find_rows_by_resolved_conditions_desc(&resolved);

        if indices.is_empty() {
            return Err(DomainError::ValueNotFound {
                miss_value: conditions[0].value.clone(),
                in_the_column: "<composite>".into(),
                reason: "No rows matched condition".into(),
            });
        }

        for i in indices.iter() {
            self.rows[*i].set_value(target_index, target_val.clone());
        }

        Ok(indices.len())
    }

    pub(crate) fn delete_row_where(
        &mut self,
        conditions: &[Condition],
    ) -> Result<usize, DomainError> {
        let resolved = self.schema.resolve_conditions(conditions)?;
        let indices = self.find_rows_by_resolved_conditions_desc(&resolved);

        if indices.is_empty() {
            return Err(DomainError::ValueNotFound {
                miss_value: conditions[0].value.clone(),
                in_the_column: "<composite>".into(),
                reason: "No rows matched condition".into(),
            });
        }

        for i in indices.iter() {
            self.rows.remove(*i);
        }

        Ok(indices.len())
    }
}

// Lookup API for application layer (read-only)
impl Table {
    pub(crate) fn select_all(&self) -> Vec<Vec<Value>> {
        self.rows
            .iter()
            .map(|row| row.get_values().clone())
            .collect()
    }

    pub(crate) fn select_where(
        &self,
        column: &str,
        value: &Value,
    ) -> Result<Vec<Vec<Value>>, DomainError> {
        let index = self.schema.validate_resolve_column(column)?;

        let result = self
            .rows
            .iter()
            .filter(|row| row.value_is_match(index, &Cmp::Eq, value))
            .map(|row| row.get_values().clone())
            .collect();

        Ok(result)
    }

    pub(crate) fn select_columns(&self, columns: &[&str]) -> Result<Vec<Vec<Value>>, DomainError> {
        //  resolve index kolom (sekali di awal)
        let indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.validate_resolve_column(name))
            .collect::<Result<_, _>>()?;

        //  ambil value sesuai index
        let result = self
            .rows
            .iter()
            .map(|row| {
                indices
                    .iter()
                    .map(|&i| row.get_values()[i].clone())
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
        //  resolve index where
        let where_index = self.schema.validate_resolve_column(column)?;

        //  resolve index projection (sekali di awal)
        let projection_indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.validate_resolve_column(name))
            .collect::<Result<_, _>>()?;

        //  filter + project
        let result = self
            .rows
            .iter()
            .filter(|row| row.value_is_match(where_index, &Cmp::Eq, value))
            .map(|row| {
                projection_indices
                    .iter()
                    .map(|&i| row.get(i).clone())
                    .collect::<Vec<Value>>()
            })
            .collect();

        Ok(result)
    }
}
