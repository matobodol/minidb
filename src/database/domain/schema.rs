use crate::database::domain::{Column, Condition, DataType, DomainError, ResolvedCondition, Value};

#[derive(Default, Debug, Clone)]
pub struct Schema {
    columns: Vec<Column>,
}

impl Schema {
    pub(super) fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub(crate) fn match_column<F>(&self, predicate: F) -> bool
    where
        F: Fn(&Column) -> bool,
    {
        self.columns.iter().any(|column| predicate(column))
    }

    pub(super) fn len(&self) -> usize {
        self.columns.len()
    }

    pub(crate) fn add_column(&mut self, columns: Vec<(&str, DataType)>) -> Result<(), DomainError> {
        let mut new_columns = Vec::with_capacity(columns.len());

        for (name, data_type) in columns {
            if self.match_column(|column| column.name() == name) {
                return Err(DomainError::DuplicateColumnName);
            }

            new_columns.push(Column::new(name, data_type));
        }

        self.columns.extend(new_columns);
        Ok(())
    }
    pub(super) fn delete(&mut self, index: usize) -> Result<usize, DomainError> {
        // validate uniqe in here. flag uniqe status pending.
        // logic: operation delete column block if flag unique is true.

        let before = self.len();
        self.columns.remove(index);

        let afected = before - self.len();
        if afected == 0 {
            return Err(DomainError::ColumnNotFound);
        }

        Ok(afected)
    }
}

// VALIDATOR
impl Schema {
    pub(crate) fn resolve_conditions(
        &self,
        conditions: &[Condition],
    ) -> Result<Vec<ResolvedCondition>, DomainError> {
        conditions
            .iter()
            .map(|c| {
                Ok(ResolvedCondition {
                    index: self.validate_resolve_column(&c.column)?,
                    cmp: c.cmp.clone(),
                    value: c.value.clone(),
                })
            })
            .collect()
    }

    /// Validates column existence and returns its index
    pub(super) fn validate_resolve_column(&self, name: &str) -> Result<usize, DomainError> {
        self.columns
            .iter()
            .position(|column| column.name() == name)
            .ok_or(DomainError::ColumnNotFound)
    }
    pub(crate) fn validate_row(&self, values: &[Value]) -> Result<(), DomainError> {
        self.validate_len(values.len())?;

        for (index, (value, column)) in values.iter().zip(self.columns().iter()).enumerate() {
            self.validate_value(index, column, value)?;
        }
        Ok(())
    }

    pub(crate) fn validate_update(&self, index: usize, value: &Value) -> Result<(), DomainError> {
        let column = self.columns.get(index).ok_or(DomainError::ColumnNotFound)?;

        self.validate_value(index, column, value)
    }
    fn validate_value(
        &self,
        index: usize,
        column: &Column,
        value: &Value,
    ) -> Result<(), DomainError> {
        match value {
            Value::Null if !column._is_nullable() => Err(DomainError::NotAllowedNull),
            _ => self.validate_type(index, column.data_type(), value),
        }
    }

    pub(crate) fn validate_len(&self, values_len: usize) -> Result<(), DomainError> {
        if values_len != self.columns.len() {
            return Err(DomainError::ColumnCountMismatch {
                expected: self.columns.len(),
                found: values_len,
            });
        }

        Ok(())
    }

    pub(crate) fn validate_type(
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
