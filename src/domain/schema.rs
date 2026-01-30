use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, Condition, Constraint, DataType, DomainError, ResolvedCondition, Value,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    columns: Vec<Column>,
}

impl Schema {
    pub(super) fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }
    pub(super) fn columns(&self) -> &[Column] {
        &self.columns
    }

    // pub(crate) fn match_column<F>(&self, predicate: F) -> bool
    // where
    //     F: Fn(&Column) -> bool,
    // {
    //     self.columns.iter().any(|column| predicate(column))
    // }

    pub(crate) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        let mut new_columns = Vec::with_capacity(columns.len());

        let mut seen = HashSet::new();

        for (name, data_type, constraint) in columns {
            if !seen.insert(name) {
                return Err(DomainError::DuplicateColumnName(name.to_string()));
            };
            new_columns.push(Column::new(name, data_type, constraint.to_vec()));
        }

        for column in self.columns() {
            if !seen.insert(column.name()) {
                return Err(DomainError::DuplicateColumnName(column.name().to_string()));
            };
        }

        self.columns.extend(new_columns);
        Ok(())
    }

    pub(super) fn remove_at(&mut self, index: usize) -> Result<usize, DomainError> {
        let before = self.columns.len();

        let is_unique = self.columns()[index].has_constraint(|c| matches!(c, Constraint::Unique));

        if is_unique {
            return Err(DomainError::BlockByConstraint(
                "Operation Was Cancelled By Constraint Unique.".to_owned(),
            ));
        }
        self.columns.remove(index);

        let afected = before - self.columns.len();
        if afected == 0 {
            return Err(DomainError::ColumnIndexNotFound(index));
        }

        Ok(afected)
    }
}

// VALIDATOR
impl Schema {
    pub(super) fn resolve_conditions(
        &self,
        conditions: &[Condition],
    ) -> Result<Vec<ResolvedCondition>, DomainError> {
        conditions
            .iter()
            .map(|cond| {
                Ok(ResolvedCondition {
                    index: self.resolve_column(&cond.column)?,
                    cmp: cond.cmp.clone(),
                    value: cond.value.clone(),
                })
            })
            .collect::<Result<_, _>>()
    }

    /// Validates column existence and returns its index
    pub(super) fn resolve_column(&self, name: &str) -> Result<usize, DomainError> {
        self.columns
            .iter()
            .position(|column| column.name() == name)
            .ok_or(DomainError::ColumnNotFound(name.to_string()))
    }
    pub(super) fn validate_row(&self, values: &[Value]) -> Result<(), DomainError> {
        self.validate_len(values.len())?;

        for (index, (value, column)) in values.iter().zip(self.columns().iter()).enumerate() {
            self.validate_type(index, column.data_type(), value)?;
        }
        Ok(())
    }

    pub(super) fn validate_update(&self, index: usize, value: &Value) -> Result<(), DomainError> {
        let column = self
            .columns
            .get(index)
            .ok_or(DomainError::ColumnIndexNotFound(index))?;

        self.validate_type(index, column.data_type(), value)
    }

    pub(super) fn validate_len(&self, values_len: usize) -> Result<(), DomainError> {
        if values_len != self.columns.len() {
            return Err(DomainError::ColumnCountMismatch {
                expected: self.columns.len(),
                found: values_len,
            });
        }

        Ok(())
    }

    pub(super) fn validate_type(
        &self,
        target_index: usize,
        data_type: &DataType,
        value: &Value,
    ) -> Result<(), DomainError> {
        if !data_type.matches(value) {
            return Err(DomainError::TypeMismatch {
                column_index: target_index,
                expected: data_type.clone(),
                found: value.clone(),
            });
        }
        Ok(())
    }
}

// VALIDATOR Constraint
// pending..
