// domain/table.rs
use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, CompareOp, Constraint, DataType, DomainError, Expr, ResolvedCompare, ResolvedExpr, Row,
    Schema, TableIndex, TableMeta, Value, compare,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    schema: Schema,
    rows: Vec<Option<Row>>, // Changed: Option for deleted rows
    meta: TableMeta,
    #[serde(skip)]
    index: TableIndex,
    deleted_count: usize, // NEW: track number of deleted rows
}

// Snapshot for transaction rollback
#[derive(Debug, Clone)]
struct TableSnapshot {
    rows: Vec<Option<Row>>,
    row_count: usize,
    deleted_count: usize,
    meta: TableMeta,
    index: TableIndex,
}

impl Table {
    pub(super) fn new() -> Self {
        Self {
            schema: Schema::new(),
            rows: Vec::new(),
            meta: TableMeta::default(),
            index: TableIndex::new(),
            deleted_count: 0,
        }
    }

    pub fn rebuild_index_after_load(&mut self) {
        // Filter out deleted rows before rebuilding
        let live_rows: Vec<&Row> = self
            .rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
            .collect();
        self.index
            .rebuild_from_rows_with_filter(&self.schema, &live_rows);
        self.deleted_count = 0;
    }

    fn create_snapshot(&self) -> TableSnapshot {
        TableSnapshot {
            rows: self.rows.clone(),
            row_count: self.rows.len(),
            deleted_count: self.deleted_count,
            meta: self.meta.clone(),
            index: self.index.clone(),
        }
    }

    fn rollback_to_snapshot(&mut self, snapshot: TableSnapshot) {
        self.rows = snapshot.rows;
        self.deleted_count = snapshot.deleted_count;
        self.meta = snapshot.meta;
        self.index = snapshot.index;
    }

    // Check if a row is alive (not deleted)
    fn is_row_alive(&self, row_idx: usize) -> bool {
        self.rows
            .get(row_idx)
            .map(|row_opt| row_opt.is_some())
            .unwrap_or(false)
    }

    // Get live row count (total - deleted)
    fn live_row_count(&self) -> usize {
        self.rows.len() - self.deleted_count
    }

    // Check if vacuum is needed
    fn should_vacuum(&self) -> bool {
        self.deleted_count > 1000
            || (self.deleted_count > 0 && self.deleted_count > self.rows.len() / 3)
    }

    // Vacuum: remove all deleted rows and rebuild indices
    fn vacuum(&mut self) {
        if self.deleted_count == 0 {
            return;
        }

        let old_len = self.rows.len();
        let mut new_rows = Vec::with_capacity(self.live_row_count());
        let mut remap = vec![None; old_len];
        let mut new_idx = 0;

        // Build new rows vector and remap table
        for (old_idx, row_opt) in self.rows.drain(..).enumerate() {
            if let Some(row) = row_opt {
                remap[old_idx] = Some(new_idx);
                new_rows.push(Some(row));
                new_idx += 1;
            }
        }

        self.rows = new_rows;

        // Update indices with new row indices
        for (_, col_index) in self.index.indices_mut() {
            let mut new_col_index = std::collections::HashMap::new();
            for (value, old_idx) in col_index.drain() {
                if let Some(new_idx) = remap.get(old_idx).and_then(|x| *x) {
                    if new_idx < self.rows.len() {
                        new_col_index.insert(value, new_idx);
                    }
                }
            }
            *col_index = new_col_index;
        }

        self.deleted_count = 0;
    }

    // Remove rows with lazy deletion
    fn remove_rows_lazy(&mut self, indices: Vec<usize>) -> usize {
        if indices.is_empty() {
            return 0;
        }

        let mut removed = 0;

        // Mark rows as deleted and remove from indices
        for &row_idx in &indices {
            if let Some(Some(row)) = self.rows.get_mut(row_idx) {
                // Remove from indices first
                for (col_idx, col) in self.schema.columns().iter().enumerate() {
                    if col.is_unique() || col.is_primary_key() {
                        let val = &row.values()[col_idx];
                        if !matches!(val, Value::Null) {
                            self.index.remove(col.name(), val, row_idx);
                        }
                    }
                }
                // Mark as deleted
                self.rows[row_idx] = None;
                removed += 1;
            }
        }

        self.deleted_count += removed;

        // Auto-vacuum if needed
        if self.should_vacuum() {
            self.vacuum();
        }

        removed
    }

    // Get live row at index (panics if deleted)
    fn get_live_row(&self, row_idx: usize) -> &Row {
        self.rows[row_idx].as_ref().expect("Row was deleted")
    }

    fn get_live_row_mut(&mut self, row_idx: usize) -> &mut Row {
        self.rows[row_idx].as_mut().expect("Row was deleted")
    }

    fn find_matching_rows(&self, resolved_expr: &ResolvedExpr) -> Vec<usize> {
        self.rows
            .iter()
            .enumerate()
            .filter(|(_, row_opt)| {
                row_opt
                    .as_ref()
                    .map_or(false, |row| Self::row_matches(row, resolved_expr))
            })
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

    fn check_uniqueness_fast(
        &self,
        column_name: &str,
        value: &Value,
        exclude_row: Option<usize>,
    ) -> bool {
        // Only consider live rows for uniqueness check
        if let Some(row_idx) = self.index.lookup(column_name, value) {
            if self.is_row_alive(row_idx) {
                return Some(row_idx) != exclude_row;
            }
        }
        false
    }

    fn rebuild_index(&mut self) {
        let live_rows: Vec<&Row> = self
            .rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
            .collect();
        self.index
            .rebuild_from_rows_with_filter(&self.schema, &live_rows);
    }
}

// COLUMN OPERATION
use std::collections::HashSet;

impl Table {
    pub(super) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        let mut has_pk = self.columns().iter().any(|c| c.is_primary_key());
        let mut has_increment = self.columns().iter().any(|c| c.is_increment());

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

            if is_increment {
                if has_increment {
                    return Err(DomainError::MultipleAutoIncrement);
                }
                has_increment = true;
                match dtype {
                    DataType::Int | DataType::Float => {}
                    _ => return Err(DomainError::InvalidAutoIncrementType),
                }
            }

            if let DataType::Enum { variants } = dtype {
                let mut uniq = HashSet::new();
                for variant in variants {
                    if !uniq.insert(variant) {
                        return Err(DomainError::DuplicateEnumVariant);
                    }
                }
                if let Some(default) = default {
                    if let Value::Enum { value } = default {
                        if !variants.contains(value) {
                            return Err(DomainError::InvalidEnumDefault);
                        }
                    } else {
                        return Err(DomainError::InvalidDefaultType);
                    }
                }
            }

            if let Some(default) = default {
                if !dtype.matches_type(default) {
                    return Err(DomainError::InvalidDefaultType);
                }
            }
        }

        let added = columns.len();
        self.schema.add_column(columns)?;

        // Reserve capacity for all rows
        for row_opt in &mut self.rows {
            if let Some(row) = row_opt {
                row.reserve(added);
                for _ in 0..added {
                    row.append(Value::Null);
                }
            }
        }

        // Add indices for new unique/primary key columns
        for col in self.schema.columns().iter().rev().take(added) {
            if col.is_unique() || col.is_primary_key() {
                self.index.ensure_index(col.name().to_string());
                let col_idx = self.schema.resolve_column(col.name()).unwrap();
                for (row_idx, row_opt) in self.rows.iter().enumerate() {
                    if let Some(row) = row_opt {
                        let val = &row.values()[col_idx];
                        if !matches!(val, Value::Null) {
                            self.index.insert(col.name(), val.clone(), row_idx);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub(super) fn delete_columns(&mut self, columns: Vec<String>) -> Result<usize, DomainError> {
        use std::collections::BTreeSet;

        let mut indexes = BTreeSet::new();

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

        for &index in indexes.iter().rev() {
            for row_opt in &mut self.rows {
                if let Some(row) = row_opt {
                    row.remove_at(index);
                }
            }
        }

        self.schema
            .remove_many(&indexes.iter().copied().collect::<Vec<_>>());

        for name in columns {
            self.meta.remove_increment(&name);
            self.index.remove_index(&name);
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

// ROW OPERATION
impl Table {
    fn build_validated_row(
        &mut self,
        inputs: Vec<Option<Value>>,
        exclude_row: Option<usize>,
    ) -> Result<Vec<Value>, DomainError> {
        let columns = self.schema.columns();
        let mut resolved: Vec<Option<Value>> = inputs.clone();

        for (idx, column) in columns.iter().enumerate() {
            if exclude_row.is_none() && resolved[idx].is_none() && column.is_increment() {
                resolved[idx] = Some(Value::Int(self.meta.next_increment(column.name())));
            }
        }

        let mut values = Vec::with_capacity(columns.len());
        let mut uniqueness_violations = Vec::new();

        for (idx, column) in columns.iter().enumerate() {
            let input = resolved[idx].take();

            let coerced = match input {
                Some(v) => column.data_type().coerce_value(v)?,
                None => {
                    if let Some(default) = column.default_value() {
                        column.data_type().coerce_value(default.clone())?
                    } else if !column.is_nullable() {
                        return Err(DomainError::NotAllowedNull);
                    } else {
                        Value::Null
                    }
                }
            };

            if (column.is_unique() || column.is_primary_key()) && !matches!(coerced, Value::Null) {
                if self.check_uniqueness_fast(column.name(), &coerced, exclude_row) {
                    uniqueness_violations.push(column.name().to_string());
                }
            }

            values.push(coerced);
        }

        if !uniqueness_violations.is_empty() {
            return Err(DomainError::NotUniqValue(uniqueness_violations.join(", ")));
        }

        Ok(values)
    }

    pub(super) fn insert_rows(
        &mut self,
        columns: Option<Vec<String>>,
        rows: Vec<Vec<Value>>,
    ) -> Result<usize, DomainError> {
        if rows.is_empty() {
            return Ok(0);
        }

        let count = rows.len();
        let indexes = match columns {
            Some(cols) => cols
                .iter()
                .map(|name| self.schema.resolve_column(name))
                .collect::<Result<Vec<_>, _>>()?,
            None => (0..self.columns().len()).collect(),
        };

        let mut validated_rows = Vec::with_capacity(rows.len());
        let mut batch_uniqueness_map = std::collections::HashMap::new();

        for (batch_idx, values) in rows.iter().enumerate() {
            if values.len() != indexes.len() {
                return Err(DomainError::ColumnValueMismatch);
            }

            let mut buffer = vec![None; self.schema.columns().len()];
            for (pos, &col_idx) in indexes.iter().enumerate() {
                if buffer[col_idx].is_some() {
                    return Err(DomainError::InsertDuplicateValuesInColumn(
                        self.schema.columns()[col_idx].name().to_string(),
                    ));
                }
                buffer[col_idx] = Some(values[pos].clone());
            }

            for (col_idx, column) in self.schema.columns().iter().enumerate() {
                if column.is_unique() || column.is_primary_key() {
                    if let Some(ref val) = buffer[col_idx] {
                        let key = (column.name().to_string(), val.clone());
                        if let Some(prev_idx) = batch_uniqueness_map.get(&key) {
                            return Err(DomainError::NotUniqValue(format!(
                                "{}: duplicate value '{}' in batch at rows {} and {}",
                                column.name(),
                                val,
                                prev_idx,
                                batch_idx
                            )));
                        }
                        batch_uniqueness_map.insert(key, batch_idx);
                    }
                }
            }

            let validated = self.build_validated_row(buffer, None)?;
            validated_rows.push(validated);
        }

        for row_values in validated_rows {
            for (index, column) in self.schema.columns().iter().enumerate() {
                if column.is_increment() {
                    if let Value::Int(v) = row_values[index] {
                        self.meta.sync_increment(column.name(), v);
                    }
                }
            }

            let row_idx = self.rows.len();

            for (idx, column) in self.schema.columns().iter().enumerate() {
                if column.is_unique() || column.is_primary_key() {
                    let val = &row_values[idx];
                    if !matches!(val, Value::Null) {
                        self.index.insert(column.name(), val.clone(), row_idx);
                    }
                }
            }

            self.rows.push(Some(Row::new(row_values)));
        }

        Ok(count)
    }

    pub(super) fn update_rows(
        &mut self,
        assignments: Vec<(String, Value)>,
        expr: &Expr,
    ) -> Result<usize, DomainError> {
        let resolved = self.schema.bind_expr(expr)?;

        let target_indices: Vec<usize> = self
            .rows
            .iter()
            .enumerate()
            .filter(|(_, row_opt)| {
                row_opt
                    .as_ref()
                    .map_or(false, |row| Self::row_matches(row, &resolved))
            })
            .map(|(i, _)| i)
            .collect();

        if target_indices.is_empty() {
            return Ok(0);
        }

        let mut assign_map = std::collections::HashMap::new();
        for (col, val) in assignments {
            let idx = self.schema.resolve_column(&col)?;
            if assign_map.insert(idx, val).is_some() {
                return Err(DomainError::DuplicateUpdateColumn);
            }
        }

        let mut pending_updates = Vec::new();

        for &row_idx in &target_indices {
            let old_row = self.get_live_row(row_idx);
            let old_values = old_row.values().to_vec();

            let inputs: Vec<Option<Value>> = self
                .schema
                .columns()
                .iter()
                .enumerate()
                .map(|(col_idx, _)| {
                    assign_map
                        .get(&col_idx)
                        .cloned()
                        .or_else(|| Some(old_values[col_idx].clone()))
                })
                .collect();

            for (&col_idx, new_val) in &assign_map {
                let column = &self.schema.columns()[col_idx];
                if (column.is_unique() || column.is_primary_key())
                    && !matches!(new_val, Value::Null)
                {
                    if self.check_uniqueness_fast(column.name(), new_val, Some(row_idx)) {
                        return Err(DomainError::NotUniqValue(column.name().to_string()));
                    }
                }
            }

            let new_row_values = self.build_validated_row(inputs, Some(row_idx))?;
            pending_updates.push((row_idx, old_values, new_row_values));
        }

        for (row_idx, old_values, new_row_values) in pending_updates {
            for (col_idx, column) in self.schema.columns().iter().enumerate() {
                if column.is_unique() || column.is_primary_key() {
                    let old_val = &old_values[col_idx];
                    if !matches!(old_val, Value::Null) {
                        self.index.remove(column.name(), old_val, row_idx);
                    }
                }
            }

            *self.get_live_row_mut(row_idx) = Row::new(new_row_values.clone());

            for (col_idx, column) in self.schema.columns().iter().enumerate() {
                if column.is_unique() || column.is_primary_key() {
                    let new_val = &new_row_values[col_idx];
                    if !matches!(new_val, Value::Null) {
                        self.index.insert(column.name(), new_val.clone(), row_idx);
                    }
                }
            }
        }

        Ok(target_indices.len())
    }

    pub(super) fn delete_rows(&mut self, expr: &Expr) -> Result<usize, DomainError> {
        let resolved = self.schema.bind_expr(expr)?;
        let indices = self.find_matching_rows(&resolved);
        Ok(self.remove_rows_lazy(indices))
    }
}

// LOOKUP API
impl Table {
    pub(super) fn lookup_all(&self) -> Vec<Vec<&Value>> {
        self.rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
            .map(|row| row.values().iter().collect())
            .collect()
    }

    fn try_index_lookup(&self, expr: &ResolvedExpr) -> Option<Vec<usize>> {
        match expr {
            ResolvedExpr::Compare(cmp) => {
                if cmp.op == CompareOp::Eq {
                    if let Some(ref value) = cmp.value {
                        let col_name = &self.schema.columns()[cmp.index].name();
                        if let Some(row_idx) = self.index.lookup(col_name, value) {
                            if self.is_row_alive(row_idx) {
                                return Some(vec![row_idx]);
                            }
                        }
                    }
                }
                None
            }
            ResolvedExpr::And(xs) => {
                let mut best: Option<Vec<usize>> = None;
                for x in xs {
                    if let Some(candidates) = self.try_index_lookup(x) {
                        match &best {
                            None => best = Some(candidates),
                            Some(existing) if candidates.len() < existing.len() => {
                                best = Some(candidates);
                            }
                            _ => {}
                        }
                    }
                }
                best
            }
            _ => None,
        }
    }

    pub(super) fn lookup_where(&self, conditions: &Expr) -> Result<Vec<Vec<&Value>>, DomainError> {
        let expr = self.schema.bind_expr(conditions)?;

        if let Some(candidates) = self.try_index_lookup(&expr) {
            let result: Vec<Vec<&Value>> = candidates
                .into_iter()
                .filter(|&row_idx| {
                    self.rows[row_idx]
                        .as_ref()
                        .map_or(false, |row| Self::row_matches(row, &expr))
                })
                .map(|row_idx| {
                    self.rows[row_idx]
                        .as_ref()
                        .unwrap()
                        .values()
                        .iter()
                        .collect()
                })
                .collect();
            return Ok(result);
        }

        let result = self
            .rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
            .filter(|row| Self::row_matches(row, &expr))
            .map(|row| row.values().iter().collect())
            .collect();

        Ok(result)
    }

    pub(super) fn lookup_where_with_limit(
        &self,
        conditions: &Expr,
        limit: usize,
    ) -> Result<Vec<Vec<&Value>>, DomainError> {
        let expr = self.schema.bind_expr(conditions)?;

        if let Some(candidates) = self.try_index_lookup(&expr) {
            let result: Vec<Vec<&Value>> = candidates
                .into_iter()
                .filter(|&row_idx| {
                    self.rows[row_idx]
                        .as_ref()
                        .map_or(false, |row| Self::row_matches(row, &expr))
                })
                .take(limit)
                .map(|row_idx| {
                    self.rows[row_idx]
                        .as_ref()
                        .unwrap()
                        .values()
                        .iter()
                        .collect()
                })
                .collect();
            return Ok(result);
        }

        let result = self
            .rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
            .filter(|row| Self::row_matches(row, &expr))
            .take(limit)
            .map(|row| row.values().iter().collect())
            .collect();

        Ok(result)
    }

    pub(super) fn lookup_columns(&self, columns: &[&str]) -> Result<Vec<Vec<&Value>>, DomainError> {
        let indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        let result = self
            .rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
            .map(|row| indices.iter().map(|&i| &row.values()[i]).collect())
            .collect();

        Ok(result)
    }

    pub(super) fn lookup_columns_where(
        &self,
        conditions: &Expr,
        columns: &[&str],
    ) -> Result<Vec<Vec<&Value>>, DomainError> {
        let projection_indices: Vec<usize> = columns
            .iter()
            .map(|name| self.schema.resolve_column(name))
            .collect::<Result<_, _>>()?;

        let expr = self.schema.bind_expr(conditions)?;

        if let Some(candidates) = self.try_index_lookup(&expr) {
            let result: Vec<Vec<&Value>> = candidates
                .into_iter()
                .filter(|&row_idx| {
                    self.rows[row_idx]
                        .as_ref()
                        .map_or(false, |row| Self::row_matches(row, &expr))
                })
                .map(|row_idx| {
                    let row = self.rows[row_idx].as_ref().unwrap();
                    projection_indices
                        .iter()
                        .map(|&i| &row.values()[i])
                        .collect()
                })
                .collect();
            return Ok(result);
        }

        let result = self
            .rows
            .iter()
            .filter_map(|row_opt| row_opt.as_ref())
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
