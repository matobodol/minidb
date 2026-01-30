use serde::{Deserialize, Serialize};

use crate::domain::{
    Condition, Constraint, DataType, DomainError, ResolvedCondition, Row, Schema, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    schema: Schema,
    rows: Vec<Row>,
}
impl Table {
    pub(crate) fn new() -> Self {
        Self {
            schema: Schema::new(),
            rows: Vec::new(),
        }
    }

    // filter: bekerja bersama (row.value_is_match())
    // cari index baris(visual: vertikal). bukan index values (visual: horizontal)
    pub(crate) fn find_rows_by_resolved_conditions(
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
        conditions.iter().all(|cond| row.value_is_match(cond))
    }
}

// COLUMN OPERATION
impl Table {
    pub(crate) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        self.schema.add_column(columns)?;

        for row in &mut self.rows {
            if row.values().len() < self.schema.columns().len() {
                row.append(Value::Absen);
            }
        }
        Ok(())
    }

    pub(crate) fn delete_column(&mut self, column: &str) -> Result<usize, DomainError> {
        let index = self.schema.resolve_column(column)?;

        let afected = self.schema.remove_at(index)?;
        self.rows
            .iter_mut()
            .for_each(|values| values.remove_at(index));

        Ok(afected)
    }
}

// ROW OPERATION FINAL
impl Table {
    pub(crate) fn insert_row(&mut self, values: &[(&str, Value)]) -> Result<(), DomainError> {
        // buffer awal: None (belum terisi)
        let mut buffer: Vec<Option<Value>> = vec![None; self.schema.columns().len()];

        // isi dari input
        for (name, value) in values {
            let index = self.schema.resolve_column(name)?;

            // cegah duplicat insert
            if buffer[index].is_some() {
                return Err(DomainError::InsertDuplicateValuesInColumn(name.to_string()));
            }

            // cek UNIQUE hanya jika kolom punya constraint UNIQUE
            let column = &self.schema.columns()[index];
            if column.has_constraint(|c| matches!(c, Constraint::Unique)) {
                let duplicated = self.rows.iter().any(|row| row.values()[index] == *value);

                if duplicated {
                    return Err(DomainError::BlockByConstraint(
                        "Value inserted not unique.".to_string(),
                    ));
                }
            }

            // validasi tipe
            self.schema.validate_update(index, value)?;
            buffer[index] = Some(value.clone());
        }

        let row_values: Vec<Value> = buffer
            .into_iter()
            .enumerate()
            .map(|(index, slot)| {
                let column = &self.schema.columns()[index];

                match slot {
                    Some(v) => Ok(v),
                    None => {
                        // 1. default?
                        if let Some(default) = column.get_constraint(|c| {
                            if let Constraint::Default(v) = c {
                                Some(v.clone())
                            } else {
                                None
                            }
                        }) {
                            return Ok(default);
                        }

                        // 2. not null?
                        if column.has_constraint(|c| matches!(c, Constraint::NotNull)) {
                            return Err(DomainError::NotAllowedNull);
                        }

                        // 3. nullable / no constraint
                        Ok(Value::Absen)
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
        to_replace: (&str, Value),
    ) -> Result<usize, DomainError> {
        let (column, value) = to_replace;
        let index = self.schema.resolve_column(column)?;

        self.schema.validate_update(index, &value)?;

        let resolved = self.schema.resolve_conditions(conditions)?;
        let indices = self.find_rows_by_resolved_conditions(&resolved);

        if indices.is_empty() {
            return Err(DomainError::InvalidCondition {
                reason: "No resolved conditions provided".into(),
            });
        }

        // ceq unique
        if self.schema.columns()[index].has_constraint(|c| matches!(c, Constraint::Unique)) {
            for (i, row) in self.rows.iter().enumerate() {
                // skip row yang memang akan diupdate
                if indices.contains(&i) {
                    continue;
                }

                if row.values()[index] == value {
                    return Err(DomainError::BlockByConstraint(
                        "Value inserted not unique.".to_string(),
                    ));
                }
            }
        }

        for i in indices.iter() {
            self.rows[*i].replace(index, value.clone());
        }

        Ok(indices.len())
    }

    pub(crate) fn delete_row_where(
        &mut self,
        conditions: &[Condition],
    ) -> Result<usize, DomainError> {
        let resolved = self.schema.resolve_conditions(conditions)?;
        let indices = self.find_rows_by_resolved_conditions(&resolved);

        if indices.is_empty() {
            return Err(DomainError::InvalidCondition {
                reason: "No resolved conditions provided".into(),
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
        self.rows.iter().map(|row| row.values().clone()).collect()
    }

    pub(crate) fn select_where(
        &self,
        condition: Condition,
    ) -> Result<Vec<Vec<Value>>, DomainError> {
        let cond = self.schema.resolve_conditions(&[condition])?;

        let result = self
            .rows
            .iter()
            .filter(|row| cond.iter().any(|c| row.value_is_match(c)))
            .map(|row| row.values().clone())
            .collect();

        Ok(result)
    }

    pub(crate) fn select_columns(&self, columns: &[&str]) -> Result<Vec<Vec<Value>>, DomainError> {
        //  resolve index kolom (sekali di awal)
        let indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        //  ambil value sesuai index
        let result = self
            .rows
            .iter()
            .map(|row| {
                indices
                    .iter()
                    .map(|&i| row.values()[i].clone())
                    .collect::<Vec<Value>>()
            })
            .collect();

        Ok(result)
    }
    pub(crate) fn select_where_columns(
        &self,
        condition: Condition,
        columns: &[&str],
    ) -> Result<Vec<Vec<Value>>, DomainError> {
        //  resolve index where

        //  resolve index projection (sekali di awal)
        let projection_indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        let cond = self.schema.resolve_conditions(&[condition])?;

        //  filter + project
        let result = self
            .rows
            .iter()
            .filter(|row| cond.iter().any(|c| row.value_is_match(c)))
            .map(|row| {
                projection_indices
                    .iter()
                    .map(|&i| row.values()[i].clone())
                    .collect::<Vec<Value>>()
            })
            .collect();

        Ok(result)
    }
}
