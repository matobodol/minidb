use crate::database::domain::{DataType, DomainError, Value};

#[derive(Debug, Clone)]

// _* definisi status rencana dipending
struct _Flag {
    // _unique: bool,
    // _increment: bool,
    _nullable: bool,
}
impl _Flag {
    fn new() -> Self {
        Self {
            // _unique: false,
            // _increment: false,
            _nullable: true, //default sementara valid untuk  row baru,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    data_type: DataType,

    // tunda optimasi dan validadi
    _flag: _Flag,
}

impl Column {
    pub(crate) fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            _flag: _Flag::new(),
        }
    }
    pub(super) fn data_type(&self) -> &DataType {
        &self.data_type
    }
    pub(crate) fn _is_nullable(&self) -> bool {
        self._flag._nullable
    }
}

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

    pub(crate) fn get_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|column| &column.name == name)
    }

    pub(super) fn len(&self) -> usize {
        self.columns.len()
    }

    pub(crate) fn add_column(&mut self, columns: Vec<(&str, DataType)>) -> Result<(), DomainError> {
        let mut new_columns = Vec::with_capacity(columns.len());

        for (name, data_type) in columns {
            if self.match_column(|column| &column.name == name) {
                return Err(DomainError::DuplicateColumnName);
            }

            new_columns.push(Column::new(name, data_type));
        }

        self.columns.extend(new_columns);
        Ok(())
    }
}

// VALIDATOR
impl Schema {
    pub(crate) fn validate_insert(&self, values: &[Value]) -> Result<(), DomainError> {
        self.validate_len(values.len())?;

        for (index, (value, column)) in values.iter().zip(self.columns().iter()).enumerate() {
            match value {
                Value::Null if !column._is_nullable() => return Err(DomainError::NotAllowedNull),
                _ => {
                    self.validate_type(index, column.data_type(), value)?;
                }
            }
        }
        Ok(())
    }

    pub(crate) fn validate_update(&self, index: usize, value: &Value) -> Result<(), DomainError> {
        let column = &self.columns[index];

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
        if !value.matches(data_type) {
            return Err(DomainError::TypeMismatch {
                column_index: target_index,
                expected: data_type.clone(),
                found: value.clone(),
            });
        }
        Ok(())
    }
}
