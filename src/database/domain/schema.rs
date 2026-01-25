use crate::database::domain::{DataType, DomainError, Value};

#[derive(Debug, Clone)]
struct Flag {
    // _unique: bool,
    _nullable: bool,
    // _increment: bool,
}
impl Flag {
    fn new() -> Self {
        Self {
            // _unique: false,
            _nullable: true,
            // _increment: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Column {
    name: String,
    data_type: DataType,
    _flag: Flag,
}

impl Column {
    pub fn new(name: impl Into<String>, data_type: DataType) -> Self {
        Self {
            name: name.into(),
            data_type,
            _flag: Flag::new(),
        }
    }
    pub(super) fn data_type(&self) -> &DataType {
        &self.data_type
    }
}

#[derive(Default, Debug, Clone)]
pub struct Schema {
    columns: Vec<Column>,
}

impl Schema {
    pub fn new(columns: Vec<Column>) -> Self {
        Self { columns }
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

    pub(crate) fn validate_row(&self, values: &[Value]) -> Result<(), DomainError> {
        if values.len() != self.columns.len() {
            return Err(DomainError::ColumnCountMismatch {
                expected: self.columns.len(),
                found: values.len(),
            });
        }

        for (index, (value, column)) in values.iter().zip(self.columns.iter()).enumerate() {
            match value {
                Value::Null => {
                    if !column._flag._nullable {
                        return Err(DomainError::NotAllowedNull);
                    }
                }
                _ => {
                    if !value.matches(column.data_type()) {
                        return Err(DomainError::TypeMismatch {
                            column_index: index,
                            expected: column.data_type().clone(),
                            found: value.clone(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

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
}
