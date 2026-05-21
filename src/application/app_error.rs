use crate::{domain::DomainError, storage::StorageError};

#[derive(Debug)]
pub enum AppError {
    InvalidSyntax,
    DatabaseAlreadyExists,
    DatabaseNotFound,
    NoDatabaseSelected,

    InvalidCommand(String),
    InvalidOperation(String),

    ConstraintViolation(String),

    NotFound(String),

    InternalError,
}

use std::fmt;

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::InvalidCommand(msg) => write!(f, "{}", msg),
            AppError::InvalidSyntax => write!(f, "invalid syntax"),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub fn map_storage_error(err: StorageError) -> AppError {
    match err {
        StorageError::DatabaseAlreadyExists => AppError::DatabaseAlreadyExists,

        StorageError::DatabaseNotFound => AppError::DatabaseNotFound,

        StorageError::NotLoaded => AppError::InvalidOperation("database not loaded".into()),

        StorageError::Io(_) => AppError::InternalError, // IO failure ≠ user fault

        StorageError::Serde(e) => AppError::InvalidOperation(format!(
            "failed to read database file (corrupted or incompatible): {}",
            e
        )),
        StorageError::Bincode(e) => AppError::InvalidOperation(format!(
            "failed to read database file (corrupted or incompatible): {}",
            e
        )),
    }
}

pub fn map_domain_error(err: DomainError) -> AppError {
    match err {
        DomainError::EmptyEnumVariant => AppError::InvalidOperation("Empty enum variant".into()),
        DomainError::InvalidEnumValue => AppError::InvalidOperation("Invalid enum value".into()),
        DomainError::DuplicateUpdateColumn => {
            AppError::InvalidOperation("Duplicate assignment Update Column".into())
        }
        DomainError::ColumnValueMismatch => {
            AppError::InvalidOperation("Column value mis match".into())
        }

        // ===== SCHEMA =====
        DomainError::TypeMismatch { .. } => {
            AppError::InvalidOperation("Inserted value has incompatible type".into())
        }

        // ===== CONSTRAINT =====
        DomainError::NotAllowedDeleteColumnPrimaryKey(col) => AppError::InvalidOperation(format!(
            "Column '{}' cannot be deleted because it has UNIQUE constraint",
            col
        )),

        DomainError::NotAllowedNull => {
            AppError::ConstraintViolation("Null value is not allowed".into())
        }

        DomainError::NotUniqValue(col) => {
            AppError::ConstraintViolation(format!("Value must be unique in column '{}'", col))
        }

        // ===== CONSTRAINT (NEW) =====
        DomainError::MultiplePrimaryKey => {
            AppError::ConstraintViolation("Multiple primary key is not allowed".into())
        }

        DomainError::InvalidPrimaryKeyNullable => {
            AppError::ConstraintViolation("Primary key cannot be nullable".into())
        }

        DomainError::MultipleAutoIncrement => {
            AppError::ConstraintViolation("Multiple auto increment column is not allowed".into())
        }

        DomainError::InvalidAutoIncrementType => AppError::ConstraintViolation(
            "Auto increment is only allowed for numeric columns".into(),
        ),

        DomainError::DuplicateEnumVariant => {
            AppError::InvalidOperation("Duplicate enum variant is not allowed".into())
        }

        DomainError::InvalidEnumDefault => {
            AppError::ConstraintViolation("Default value is not part of enum variants".into())
        }

        DomainError::InvalidDefaultType => {
            AppError::ConstraintViolation("Default value does not match column type".into())
        }

        // ===== ROW =====
        DomainError::InsertDuplicateValuesInColumn(col) => {
            AppError::ConstraintViolation(format!("Duplicate value in column '{}'", col))
        }

        // ===== TABLE =====
        DomainError::DuplicateTableName => {
            AppError::InvalidOperation("Table already exists".into())
        }

        DomainError::TableNotFound(name) => {
            AppError::NotFound(format!("Table '{}' not found", name))
        }

        // ===== COLUMN =====
        DomainError::ColumnNotFound(name) => {
            AppError::NotFound(format!("Column '{}' not found", name))
        }

        DomainError::DuplicateColumnName(name) => {
            AppError::InvalidOperation(format!("Column '{}' already exists", name))
        }
    }
}
