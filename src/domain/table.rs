use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, CompareOp, Constraint, DataType, DomainError, Expr, ResolvedCompare, ResolvedExpr, Row,
    Schema, TableMeta, Value, compare,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    schema: Schema,
    rows: Vec<Row>,
    meta: TableMeta,
}
impl Table {
    pub(super) fn new() -> Self {
        Self {
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
            .map(|(_, row)| row)
            .collect();

        indices.len()
    }

    fn find_matching_rows(&self, resolved_expr: &ResolvedExpr) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row)| Self::row_matches(row, resolved_expr))
            .map(|(i, _)| i)
            .collect()
    }

    fn row_matches(row: &Row, expr: &ResolvedExpr) -> bool {
        match expr {
            ResolvedExpr::Compare(cmp) => Self::compare_row(row, cmp),

            ResolvedExpr::And(xs) => xs.iter().all(|x| Self::row_matches(row, x)),

            ResolvedExpr::Or(xs) => xs.iter().any(|x| Self::row_matches(row, x)),

            ResolvedExpr::Not(inner) => !Self::row_matches(row, inner),
        }
    }

    fn compare_row(row: &Row, cmp: &ResolvedCompare) -> bool {
        let selected = &row.values()[cmp.index];

        match cmp.op {
            CompareOp::IsNull => matches!(selected, Value::Null),

            CompareOp::IsNotNull => !matches!(selected, Value::Null),

            _ => {
                let Some(ref value) = cmp.value else {
                    return false;
                };

                compare(selected, &cmp.op, value)
            }
        }
    }
}

// COLUMN OPERATION
use std::collections::HashSet;

impl Table {
    pub(super) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        // =====================
        // GLOBAL CHECK
        // =====================
        let mut has_pk = self.columns().iter().any(|c| c.is_primary_key());

        let mut has_increment = self.columns().iter().any(|c| c.is_increment());

        // =====================
        // VALIDATE NEW COLUMNS
        // =====================
        for (_, dtype, constraints) in &columns {
            let is_pk = constraints
                .iter()
                .any(|c| matches!(c, Constraint::PrimaryKey));

            let is_increment = constraints
                .iter()
                .any(|c| matches!(c, Constraint::Increment));

            let default = constraints.iter().find_map(|c| {
                if let Constraint::Default(v) = c {
                    Some(v)
                } else {
                    None
                }
            });

            // PRIMARY KEY
            if is_pk {
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

            // AUTO INCREMENT
            if is_increment {
                if has_increment {
                    return Err(DomainError::MultipleAutoIncrement);
                }

                has_increment = true;

                match dtype {
                    DataType::Int | DataType::Float => {}
                    _ => {
                        return Err(DomainError::InvalidAutoIncrementType);
                    }
                }
            }

            // ENUM VALIDATION
            if let DataType::Enum { variants } = dtype {
                let mut uniq = HashSet::new();

                for variant in variants {
                    if !uniq.insert(variant) {
                        return Err(DomainError::DuplicateEnumVariant);
                    }
                }

                if let Some(Value::Enum { value }) = default {
                    if !variants.contains(value) {
                        return Err(DomainError::InvalidEnumDefault);
                    }
                }
            }

            // DEFAULT TYPE CHECK
            if let Some(default) = default {
                if !dtype.matches_type(default) {
                    return Err(DomainError::InvalidDefaultType);
                }
            }
        }

        // =====================
        // APPLY
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

    pub(super) fn delete_columns(&mut self, columns: Vec<String>) -> Result<usize, DomainError> {
        use std::collections::BTreeSet;

        let mut indexes = BTreeSet::new();

        // =====================
        // VALIDATE + RESOLVE
        // =====================
        for name in &columns {
            let index = self.schema.resolve_column(name)?;

            let column = &self.schema.columns()[index];

            if column.is_primary_key() {
                return Err(DomainError::NotAllowedDeleteColumnPrimaryKey(
                    name.to_string(),
                ));
            }

            indexes.insert(index);
        }

        // =====================
        // MUTATE ROWS
        // =====================
        for &index in indexes.iter().rev() {
            for row in &mut self.rows {
                row.remove_at(index);
            }
        }

        // =====================
        // MUTATE SCHEMA
        // =====================
        self.schema
            .remove_many(&indexes.iter().copied().collect::<Vec<_>>());

        // =====================
        // META CLEANUP
        // =====================
        for name in columns {
            self.meta.remove_increment(&name);
        }

        Ok(indexes.len())
    }

    pub(super) fn columns(&self) -> &[Column] {
        self.schema.columns()
    }

    pub(super) fn columns_selected(&self, names: &[&str]) -> Result<Vec<&Column>, DomainError> {
        names
            .iter()
            .map(|name| {
                let idx = self.schema.resolve_column(name)?;
                Ok(&self.columns()[idx])
            })
            .collect()
    }
}

// ROW OPERATION FINAL
impl Table {
    pub(super) fn insert(
        &mut self,
        columns: Option<Vec<String>>,
        rows: Vec<Vec<Value>>,
    ) -> Result<usize, DomainError> {
        let count = rows.len();

        let indexes = match columns {
            Some(cols) => cols
                .iter()
                .map(|name| self.schema.resolve_column(name))
                .collect::<Result<Vec<_>, _>>()?,
            None => (0..self.columns().len()).collect(),
        };

        for values in rows {
            if values.len() != indexes.len() {
                return Err(DomainError::ColumnValueMismatch);
            }

            let pairs = indexes.iter().copied().zip(values.into_iter()).collect();

            self.insert_row(pairs)?;
        }

        Ok(count)
    }

    pub(super) fn insert_row(&mut self, values: Vec<(usize, Value)>) -> Result<(), DomainError> {
        let columns = self.schema.columns();

        let mut buffer: Vec<Option<Value>> = vec![None; columns.len()];

        for (index, value) in values {
            if buffer[index].is_some() {
                return Err(DomainError::InsertDuplicateValuesInColumn(
                    columns[index].name().to_string(),
                ));
            }

            buffer[index] = Some(value);
        }

        let row_values: Vec<Value> = columns
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let mut input = buffer[index].take();

                if input.is_none() && column.is_increment() {
                    input = Some(Value::Int(self.meta.next_increment(column.name())));
                }

                let existing = self.rows.iter().map(|r| &r.values()[index]);

                column.enforce(input, existing)
            })
            .collect::<Result<_, _>>()?;

        for (index, column) in columns.iter().enumerate() {
            if column.is_increment() {
                if let Value::Int(v) = row_values[index] {
                    self.meta.sync_increment(column.name(), v);
                }
            }
        }

        self.rows.push(Row::new(row_values));

        Ok(())
    }

    pub(super) fn update_rows(
        &mut self,
        assignments: Vec<(String, Value)>,
        expr: &Expr,
    ) -> Result<usize, DomainError> {
        let columns = self.schema.columns();

        // resolve where
        let resolved = self.schema.bind_expr(expr)?;

        let target_indices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row)| Self::row_matches(row, &resolved))
            .map(|(i, _)| i)
            .collect();

        if target_indices.is_empty() {
            return Ok(0);
        }

        // assignment map
        let mut assign_map = std::collections::HashMap::new();

        for (col, val) in assignments {
            let idx = self.schema.resolve_column(&col)?;

            if assign_map.insert(idx, val).is_some() {
                return Err(DomainError::DuplicateUpdateColumn);
            }
        }

        let snapshot = &self.rows;
        let mut pending_updates = Vec::new();

        for &row_idx in &target_indices {
            let old_row = &snapshot[row_idx];

            let new_row_values: Vec<Value> = columns
                .iter()
                .enumerate()
                .map(|(col_idx, column)| {
                    let input = assign_map
                        .get(&col_idx)
                        .cloned()
                        .or_else(|| Some(old_row.values()[col_idx].clone()));

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

        for (row_idx, row) in pending_updates {
            self.rows[row_idx] = Row::new(row);
        }

        Ok(target_indices.len())
    }

    pub(super) fn delete_rows(&mut self, expr: &Expr) -> Result<usize, DomainError> {
        let resolved = self.schema.bind_expr(expr)?;
        let indices = self.find_matching_rows(&resolved);

        Ok(self.remove_rows(indices))
    }
}

// Lookup API for application layer (read-only)
impl Table {
    pub(super) fn lookup_all(&self) -> Vec<Vec<&Value>> {
        self.rows
            .iter()
            .map(|row| row.values().iter().collect())
            .collect()
    }
}

impl Table {
    pub(super) fn lookup_where(&self, conditions: &Expr) -> Result<Vec<Vec<&Value>>, DomainError> {
        let expr = self.schema.bind_expr(conditions)?;

        let result = self
            .rows
            .iter()
            .filter(|row| Self::row_matches(row, &expr))
            .map(|row| row.values().iter().collect())
            .collect();

        Ok(result)
    }
}

impl Table {
    pub(super) fn lookup_columns(&self, columns: &[&str]) -> Result<Vec<Vec<&Value>>, DomainError> {
        let indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        let result = self
            .rows
            .iter()
            .map(|row| indices.iter().map(|&i| &row.values()[i]).collect())
            .collect();

        Ok(result)
    }
}

impl Table {
    pub(super) fn lookup_columns_where(
        &self,
        conditions: &Expr,
        columns: &[&str],
    ) -> Result<Vec<Vec<&Value>>, DomainError> {
        // resolve projection
        let projection_indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        // bind filter
        let expr = self.schema.bind_expr(conditions)?;

        // filter + project
        let result = self
            .rows
            .iter()
            .filter(|row| Self::row_matches(row, &expr))
            .map(|row| {
                projection_indices
                    .iter()
                    .map(|&i| &row.values()[i])
                    .collect()
            })
            .collect();

        Ok(result)
    }
}
