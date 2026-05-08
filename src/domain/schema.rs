use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::domain::{Cmp, Column, Condition, Constraint, DataType, DomainError, ResolvedCondition};

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

    pub(super) fn remove_at(&mut self, index: usize) {
        self.columns.remove(index);
    }
}

// VALIDATOR
impl Schema {
    pub(super) fn bind_conditions(
        &self,
        conditions: &[Condition],
    ) -> Result<Vec<ResolvedCondition>, DomainError> {
        conditions
            .iter()
            .map(|cond| {
                let index = self.resolve_column(&cond.column)?;
                let column = &self.columns[index];

                let value = match cond.cmp {
                    Cmp::IsNull | Cmp::IsNotNull => {
                        if cond.value.is_some() {
                            return Err(DomainError::InvalidOperation(
                                "IS NULL should not have value".into(),
                            ));
                        }
                        None
                    }

                    _ => {
                        let raw = cond.value.clone().ok_or(DomainError::InvalidOperation(
                            "missing value in condition".into(),
                        ))?;

                        Some(column.data_type().coerce_value(raw)?)
                    }
                };

                Ok(ResolvedCondition {
                    index,
                    cmp: cond.cmp.clone(),
                    value,
                })
            })
            .collect()
    }

    /// Validates column existence and returns its index
    pub(super) fn resolve_column(&self, name: &str) -> Result<usize, DomainError> {
        self.columns
            .iter()
            .position(|column| column.name() == name)
            .ok_or(DomainError::ColumnNotFound(name.to_string()))
    }

    // pub(super) fn validate_len(&self, values_len: usize) -> Result<(), DomainError> {
    //     if values_len != self.columns.len() {
    //         return Err(DomainError::ColumnCountMismatch {
    //             expected: self.columns.len(),
    //             found: values_len,
    //         });
    //     }
    //
    //     Ok(())
    // }

    // pub(super) fn validate_row(&self, values: &[Value]) -> Result<(), DomainError> {
    //     // tetap penting
    //     self.validate_len(values.len())?;
    //
    //     for (value, column) in values.iter().zip(self.columns().iter()) {
    //         // hanya constraint, bukan type lagi
    //
    //         // NOT NULL / PK
    //         if matches!(value, Value::Null)
    //             && column
    //                 .has_constraint(|c| matches!(c, Constraint::NotNull | Constraint::PrimaryKey))
    //         {
    //             return Err(DomainError::NotAllowedNull);
    //         }
    //
    //         if !column.data_type().matches_type(value) {
    //             return Err(DomainError::TypeMismatch);
    //         }
    //     }
    //
    //     Ok(())
    // }
    // pub(super) fn validate_row(&self, values: &[Value]) -> Result<(), DomainError> {
    //     self.validate_len(values.len())?;
    //
    //     for (index, (value, column)) in values.iter().zip(self.columns().iter()).enumerate() {
    //         self.validate_type(index, column.data_type(), value)?;
    //     }
    //     Ok(())
    // }

    // pub(super) fn validate_update(&self, index: usize, value: &Value) -> Result<(), DomainError> {
    //     let column = self
    //         .columns
    //         .get(index)
    //         .ok_or(DomainError::ColumnIndexNotFound(index))?;
    //
    //     self.validate_type(index, column.data_type(), value)
    // }
    //
    // pub(super) fn validate_type(
    //     &self,
    //     target_index: usize,
    //     data_type: &DataType,
    //     value: &Value,
    // ) -> Result<(), DomainError> {
    //     if !data_type.matches_value(value) {
    //         return Err(DomainError::TypeMismatch);
    //     }
    //     Ok(())
    // }
}
