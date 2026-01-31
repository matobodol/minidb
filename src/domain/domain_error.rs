use crate::domain::{DataType, Value};

#[derive(Debug)]
pub enum DomainError {
    // SCHEMA
    TypeMismatch {
        column_index: usize,
        expected: DataType,
        found: Value,
    },
    ColumnCountMismatch {
        expected: usize,
        found: usize,
    },

    // CONSTRAINT
    ConstrainUniqeAlreadyExist,
    NotAllowedDeleteColumnUniq(String),
    NotAllowedNull,
    NotUniqValue(String),

    // ROW
    ValueNotFound {
        miss_value: Value,
        in_the_column: String,
        reason: String,
    },
    InsertDuplicateValuesInColumn(String),
    InvalidCondition {
        reason: String,
    },

    // TABLE
    DuplicateTableName,
    TableNotFound(String),

    // COLUMN
    ColumnIndexNotFound(usize),
    ColumnNotFound(String),
    DuplicateColumnName(String),
}
