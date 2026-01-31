#[derive(Debug)]
pub enum AppError {
    DatabaseAlreadyExists,
    DatabaseNotFound,
    NoDatabaseSelected,

    InvalidCommand(String),
    InvalidOperation(String),

    ConstraintViolation(String),

    NotFound(String),

    InternalError,
}

use crate::{domain::DomainError, storage::StorageError};
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
    }
}

pub fn map_domain_error(err: DomainError) -> AppError {
    match err {
        // ===== SCHEMA =====
        DomainError::TypeMismatch { .. } => {
            AppError::InvalidOperation("Inserted value has incompatible type".into())
        }

        DomainError::ColumnCountMismatch { .. } => {
            AppError::InvalidOperation("Number of values does not match table schema".into())
        }

        // ===== CONSTRAINT =====
        DomainError::NotAllowedDeleteColumnUniq(col) => AppError::InvalidOperation(format!(
            "Column '{}' cannot be deleted because it has UNIQUE constraint",
            col
        )),

        DomainError::NotAllowedNull => {
            AppError::ConstraintViolation("Null value is not allowed".into())
        }

        DomainError::NotUniqValue(col) => {
            AppError::ConstraintViolation(format!("Value must be unique in column '{}'", col))
        }
        DomainError::ConstrainUniqeAlreadyExist => {
            AppError::ConstraintViolation("Column with unique is already exist.".into())
        }

        // ===== ROW =====
        DomainError::ValueNotFound { .. } => {
            AppError::NotFound("Requested value was not found".into())
        }

        DomainError::InsertDuplicateValuesInColumn(col) => {
            AppError::ConstraintViolation(format!("Duplicate value in column '{}'", col))
        }

        DomainError::InvalidCondition { reason } => AppError::InvalidCommand(reason),

        // ===== TABLE =====
        DomainError::DuplicateTableName => {
            AppError::InvalidOperation("Table already exists".into())
        }

        DomainError::TableNotFound(name) => {
            AppError::NotFound(format!("Table '{}' not found", name))
        }

        // ===== COLUMN =====
        DomainError::ColumnIndexNotFound(_) => AppError::InternalError,

        DomainError::ColumnNotFound(name) => {
            AppError::NotFound(format!("Column '{}' not found", name))
        }

        DomainError::DuplicateColumnName(name) => {
            AppError::InvalidOperation(format!("Column '{}' already exists", name))
        }
    }
}
