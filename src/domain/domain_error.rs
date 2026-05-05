use crate::domain::Value;

#[derive(Debug)]
pub enum DomainError {
    InvalidOperation(String),
    // SCHEMA
    TypeMismatch,
    ColumnCountMismatch {
        expected: usize,
        found: usize,
    },

    // CONSTRAINT
    MultiplePrimaryKey,
    InvalidPrimaryKeyNullable,
    MultipleAutoIncrement,
    InvalidAutoIncrementType,
    DuplicateEnumVariant,
    InvalidEnumDefault,
    InvalidDefaultType,
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
    ColumnValueMismatch,
}
