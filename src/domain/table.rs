use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, Condition, Constraint, DataType, DomainError, ResolvedCondition, Row, Schema, Value,
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

    fn remove_rows_by_indices(&mut self, mut indices: Vec<usize>) -> usize {
        if indices.is_empty() {
            return 0;
        }

        // penting: hapus dari belakang
        indices.sort_unstable_by(|a, b| b.cmp(a));

        for &i in &indices {
            self.rows.remove(i);
        }

        indices.len()
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
        let mut found_uniq: bool = true;

        for uniq in self.columns() {
            if uniq.has_constraint(|c| matches!(c, Constraint::Unique)) {
                found_uniq = true && found_uniq;
            }
        }
        for (_, _, c) in &columns {
            if c.iter().any(|c| matches!(c, Constraint::Unique)) && found_uniq {
                return Err(DomainError::ConstrainUniqeAlreadyExist);
            }
        }

        self.schema.add_column(columns)?;

        for row in &mut self.rows {
            if row.values().len() < self.schema.columns().len() {
                row.append(Value::Absen(false));
            }
        }
        Ok(())
    }

    pub(crate) fn delete_column(&mut self, columns: Vec<String>) -> Result<usize, DomainError> {
        let mut indexes = Vec::new();

        // VALIDASI + RESOLVE
        for name in &columns {
            let index = self.schema.resolve_column(name)?;
            let column = &self.schema.columns()[index];

            if column.has_constraint(|c| matches!(c, Constraint::Unique)) {
                return Err(DomainError::NotAllowedDeleteColumnUniq(name.to_string()));
            }

            indexes.push(index);
        }

        indexes.sort();
        indexes.dedup();
        indexes.reverse();

        // MUTASI
        for &index in &indexes {
            self.schema.remove_at(index);
            self.rows.iter_mut().for_each(|row| row.remove_at(index));
        }

        Ok(indexes.len())
    }

    pub(super) fn columns(&self) -> &[Column] {
        self.schema.columns()
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
                    return Err(DomainError::NotUniqValue(
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
                        Ok(Value::Absen(false))
                    }
                }
            })
            .collect::<Result<_, _>>()?;

        self.schema.validate_row(&row_values)?;
        self.rows.push(Row::new(row_values));
        Ok(())
    }
    pub(crate) fn update_where(
        &mut self,
        conditions: &[Condition],
        assignments: &[(String, Value)],
    ) -> Result<usize, DomainError> {
        if assignments.is_empty() {
            return Ok(0);
        }

        // 1. resolve kondisi
        let resolved = self.schema.resolve_conditions(conditions)?;
        let target_rows = self.find_rows_by_resolved_conditions(&resolved);

        // 2. tidak ada row cocok → valid, bukan error
        if target_rows.is_empty() {
            return Ok(0);
        }

        // 3. resolve kolom & validasi value
        let mut resolved_updates = Vec::new();
        for (col, value) in assignments {
            let index = self.schema.resolve_column(col)?;
            self.schema.validate_update(index, value)?;
            resolved_updates.push((index, value.clone()));
        }

        // 4. cek UNIQUE constraint
        for (col_index, new_value) in &resolved_updates {
            let column = &self.schema.columns()[*col_index];

            if column.has_constraint(|c| matches!(c, Constraint::Unique)) {
                for (row_idx, row) in self.rows.iter().enumerate() {
                    // skip row yang memang akan diupdate
                    if target_rows.contains(&row_idx) {
                        continue;
                    }

                    if row.values()[*col_index] == *new_value {
                        return Err(DomainError::NotUniqValue(
                            "Value inserted not unique.".to_string(),
                        ));
                    }
                }
            }
        }

        // 5. apply update
        for row_idx in &target_rows {
            let row = &mut self.rows[*row_idx];
            for (col_index, value) in &resolved_updates {
                row.replace(*col_index, value.clone());
            }
        }

        Ok(target_rows.len())
    }

    pub(crate) fn delete_row(&mut self, conditions: &[Condition]) -> Result<usize, DomainError> {
        let resolved = self.schema.resolve_conditions(conditions)?;
        let indices = self.find_rows_by_resolved_conditions(&resolved);

        Ok(self.remove_rows_by_indices(indices))
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
