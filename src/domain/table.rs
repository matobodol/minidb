use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, CompareOp, Constraint, DataType, DomainError, Expr, ResolvedCompare, ResolvedExpr, Row,
    Schema, TableMeta, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    id_uniq: Option<String>,
    schema: Schema,
    rows: Vec<Row>,
    meta: TableMeta,
}
impl Table {
    pub(crate) fn new() -> Self {
        Self {
            id_uniq: None,
            schema: Schema::new(),
            rows: Vec::new(),
            meta: TableMeta::default(),
        }
    }

    fn remove_rows(&mut self, indices: Vec<usize>) -> usize {
        if indices.is_empty() {
            return 0;
        }

        let mut keep = vec![true; self.rows.len()];
        for &i in &indices {
            keep[i] = false;
        }

        self.rows = self
            .rows
            .drain(..)
            .enumerate()
            .filter(|(i, _)| keep[*i])
            .map(|(_, v)| v)
            .collect();

        indices.len()
    }

    // filter: bekerja bersama (row.value_is_match())
    // cari index baris(visual: vertikal). bukan index values (visual: horizontal)
    pub(crate) fn find_matching_rows(&self, resolvedexpr: &ResolvedExpr) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.row_matches(row, resolvedexpr))
            .map(|(i, _)| i)
            // .rev()
            .collect::<Vec<_>>()
    }

    pub(crate) fn row_matches(&self, row: &Row, expr: &ResolvedExpr) -> bool {
        match expr {
            ResolvedExpr::Compare(cmp) => Self::compare_row(row, cmp),

            ResolvedExpr::And(xs) => xs.iter().all(|x| self.row_matches(row, x)),

            ResolvedExpr::Or(xs) => xs.iter().any(|x| self.row_matches(row, x)),

            ResolvedExpr::Not(inner) => !self.row_matches(row, inner),
        }
    }

    fn compare_row(row: &Row, cmp: &ResolvedCompare) -> bool {
        let Some(selected) = row.values().get(cmp.index) else {
            return false;
        };

        match cmp.op {
            CompareOp::IsNull => {
                matches!(selected, Value::Null)
            }

            CompareOp::IsNotNull => !matches!(selected, Value::Null),

            _ => {
                let Some(ref value) = cmp.value else {
                    return false;
                };

                selected.compare(&cmp.op, value)
            }
        }
    }
}

// COLUMN OPERATION
impl Table {
    pub(crate) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        // =====================
        // GLOBAL CHECK (existing schema)
        // =====================
        let mut has_pk = self
            .columns()
            .iter()
            .any(|c| c.has_constraint(|c| matches!(c, Constraint::PrimaryKey)));

        let mut has_increment = self
            .columns()
            .iter()
            .any(|c| c.has_constraint(|c| matches!(c, Constraint::Increment)));

        for (_name, dtype, constraints) in &columns {
            // =====================
            // PRIMARY KEY
            // =====================
            if constraints
                .iter()
                .any(|c| matches!(c, Constraint::PrimaryKey))
            {
                if has_pk {
                    return Err(DomainError::MultiplePrimaryKey);
                }
                has_pk = true;

                if constraints
                    .iter()
                    .any(|c| matches!(c, Constraint::Nullable))
                {
                    return Err(DomainError::InvalidPrimaryKeyNullable);
                }
            }

            // =====================
            // INCREMENT
            // =====================
            if constraints
                .iter()
                .any(|c| matches!(c, Constraint::Increment))
            {
                if has_increment {
                    return Err(DomainError::MultipleAutoIncrement);
                }
                has_increment = true;

                match dtype {
                    DataType::Int | DataType::Float => {}
                    _ => return Err(DomainError::InvalidAutoIncrementType),
                }
            }

            // =====================
            // ENUM VALIDATION
            // =====================
            if let DataType::Enum { variants } = dtype {
                // unique variants
                let mut uniq = std::collections::HashSet::new();
                for v in variants {
                    if !uniq.insert(v) {
                        return Err(DomainError::DuplicateEnumVariant);
                    }
                }

                // default harus ada di enum
                if let Some(Constraint::Default(Value::Enum { value })) = constraints
                    .iter()
                    .find(|c| matches!(c, Constraint::Default(_)))
                {
                    if !variants.contains(value) {
                        return Err(DomainError::InvalidEnumDefault);
                    }
                }
            }

            // =====================
            // DEFAULT TYPE CHECK
            // =====================
            if let Some(Constraint::Default(val)) = constraints
                .iter()
                .find(|c| matches!(c, Constraint::Default(_)))
            {
                if !dtype.matches_type(val) {
                    return Err(DomainError::InvalidDefaultType);
                }
            }
        }

        // =====================
        // APPLY (kalau semua valid)
        // =====================
        let added = columns.len();

        self.schema.add_column(columns)?;

        for row in &mut self.rows {
            for _ in 0..added {
                row.append(Value::Null);
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
        // hapus meta
        for name in columns {
            self.meta.remove_increment(&name);
        }

        Ok(indexes.len())
    }

    pub(super) fn columns(&self) -> &[Column] {
        self.schema.columns()
    }

    pub(crate) fn columns_selected(&self, columns: &[&str]) -> Result<Vec<Column>, DomainError> {
        let mut selected = Vec::new();
        for name in columns {
            let index = self.schema.resolve_column(name)?;
            selected.push(self.columns()[index].clone());
        }
        Ok(selected)
    }
}

// ROW OPERATION FINAL
impl Table {
    pub fn insert(
        &mut self,
        columns: Option<Vec<String>>,
        rows: Vec<Vec<Value>>,
    ) -> Result<usize, DomainError> {
        let count = rows.len();

        for values in rows {
            let columns = columns.as_ref();
            let schema_columns = self.columns();

            let pairs: Vec<(String, Value)> = match columns {
                Some(cols) => {
                    if cols.len() != values.len() {
                        return Err(DomainError::ColumnValueMismatch);
                    }

                    cols.iter().cloned().zip(values.into_iter()).collect()
                }

                None => {
                    if values.len() != schema_columns.len() {
                        return Err(DomainError::ColumnValueMismatch);
                    }

                    schema_columns
                        .iter()
                        .map(|c| c.name().to_string())
                        .zip(values.into_iter())
                        .collect()
                }
            };

            self.insert_row(pairs)?;
        }
        Ok(count)
    }

    pub(crate) fn insert_row(&mut self, values: Vec<(String, Value)>) -> Result<(), DomainError> {
        let columns = self.schema.columns();
        let mut buffer: Vec<Option<Value>> = vec![None; columns.len()];

        // =====================
        // MAP INPUT → BUFFER
        // =====================
        for (name, value) in values {
            let index = self.schema.resolve_column(&name)?;

            if buffer[index].is_some() {
                return Err(DomainError::InsertDuplicateValuesInColumn(name.to_string()));
            }

            buffer[index] = Some(value);
        }

        // =====================
        // BUILD ROW (ENFORCE)
        // =====================
        let row_values: Vec<Value> = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let mut input = buffer[index].clone();

                // =====================
                // INCREMENT
                // =====================
                if input.is_none() && column.is_increment() {
                    input = Some(Value::Int(self.meta.next_increment(column.name())));
                }

                // iterator untuk UNIQUE check (optional dipakai di enforce)
                let existing_iter = self.rows.iter().map(|r| &r.values()[index]);

                column.enforce(input, existing_iter)
            })
            .collect::<Result<_, _>>()?;

        // =====================
        // SYNC INCREMENT
        // =====================
        for (index, column) in columns.iter().enumerate() {
            if column.is_increment() {
                if let Value::Int(v) = row_values[index] {
                    self.meta.sync_increment(column.name(), v);
                }
            }
        }

        // =====================
        // INSERT
        // =====================
        self.rows.push(Row::new(row_values));

        Ok(())
    }

    pub(crate) fn update_rows(
        &mut self,
        assignments: Vec<(String, Value)>,
        expr: &Expr,
    ) -> Result<usize, DomainError> {
        let columns = self.schema.columns().to_vec();

        // =====================
        // RESOLVE CONDITIONS
        // =====================
        let resolved = self.schema.bind_expr(expr)?;

        // =====================
        // FIND TARGET ROWS
        // =====================
        let target_indices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| self.row_matches(row, &resolved))
            .map(|(i, _)| i)
            .collect();

        if target_indices.is_empty() {
            return Ok(0);
        }

        // =====================
        // PREPARE ASSIGNMENT MAP
        // =====================
        let mut assign_map = std::collections::HashMap::new();

        for (col, val) in assignments {
            let idx = self.schema.resolve_column(&col)?;
            assign_map.insert(idx, val);
        }

        // =====================
        // BUILD NEW ROWS (NO COMMIT YET)
        // =====================
        let snapshot = &self.rows;
        let mut pending_updates = Vec::new();

        for &row_idx in &target_indices {
            let old_row = &snapshot[row_idx];

            // clone current row into mutable buffer
            let mut buffer: Vec<Option<Value>> =
                old_row.values().iter().cloned().map(Some).collect();

            // apply assignments
            for (&col_idx, val) in &assign_map {
                buffer[col_idx] = Some(val.clone());
            }

            // validate full row
            let new_row_values: Vec<Value> = columns
                .iter()
                .enumerate()
                .map(|(col_idx, column)| {
                    let input = buffer[col_idx].take();

                    let existing_iter = snapshot
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| *i != row_idx)
                        .map(|(_, r)| &r.values()[col_idx]);

                    column.enforce(input, existing_iter)
                })
                .collect::<Result<_, _>>()?;

            pending_updates.push((row_idx, new_row_values));
        }

        // =====================
        // COMMIT
        // =====================
        for (row_idx, row) in pending_updates {
            self.rows[row_idx] = Row::new(row);
        }

        Ok(target_indices.len())
    }

    pub(crate) fn delete_rows(&mut self, expr: &Expr) -> Result<usize, DomainError> {
        let resolved = self.schema.bind_expr(expr)?;
        let indices = self.find_matching_rows(&resolved);

        Ok(self.remove_rows(indices))
    }
}

// Lookup API for application layer (read-only)
impl Table {
    pub(crate) fn select_all(&self) -> Vec<Vec<String>> {
        self.rows
            .iter()
            .map(|row| row.values().iter().map(|v| v.to_display_str()).collect())
            .collect()
    }

    pub(crate) fn select_where(&self, conditions: &Expr) -> Result<Vec<Vec<String>>, DomainError> {
        let expr = self.schema.bind_expr(conditions)?;

        let result = self
            .rows
            .iter()
            .filter(|row| self.row_matches(row, &expr))
            .map(|row| row.values().iter().map(|v| v.to_display_str()).collect())
            .collect();

        Ok(result)
    }

    pub(crate) fn select_columns(&self, columns: &[&str]) -> Result<Vec<Vec<String>>, DomainError> {
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
                    .map(|&i| row.values()[i].to_display_str())
                    .collect::<Vec<String>>()
            })
            .collect();

        Ok(result)
    }

    pub(crate) fn select_columns_where(
        &self,
        conditions: &Expr,
        columns: &[&str],
    ) -> Result<Vec<Vec<String>>, DomainError> {
        // =====================
        // RESOLVE PROJECTION
        // =====================
        let projection_indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        // =====================
        // BIND WHERE EXPR
        // =====================
        let expr = self.schema.bind_expr(conditions)?;

        // =====================
        // FILTER + PROJECT
        // =====================
        let result = self
            .rows
            .iter()
            .filter(|row| self.row_matches(row, &expr))
            .map(|row| {
                projection_indices
                    .iter()
                    .map(|&i| row.values()[i].to_display_str())
                    .collect::<Vec<String>>()
            })
            .collect();

        Ok(result)
    }
}
