use crate::domain::DomainError;

#[derive(Debug)]
pub enum AppError {
    Domain(DomainError),

    DatabaseAlreadyExists,
    DatabaseNotFound,
    NoDatabaseSelected,
}
