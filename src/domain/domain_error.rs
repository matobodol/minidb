#[derive(Debug)]
pub enum DomainError {
    InvalidOperation(String),
    // SCHEMA
    TypeMismatch,

    // CONSTRAINT
    MultiplePrimaryKey,
    InvalidPrimaryKeyNullable,
    MultipleAutoIncrement,
    InvalidAutoIncrementType,
    DuplicateEnumVariant,
    InvalidEnumDefault,
    InvalidDefaultType,
    NotAllowedDeleteColumnPrimaryKey(String),
    NotAllowedNull,
    NotUniqValue(String),

    // ROW
    InsertDuplicateValuesInColumn(String),

    // TABLE
    DuplicateTableName,
    TableNotFound(String),

    // COLUMN
    ColumnNotFound(String),
    DuplicateColumnName(String),
    DuplicateUpdateColumn,
    ColumnValueMismatch,
    EmptyEnumVariant,
    InvalidEnumValue,
}
