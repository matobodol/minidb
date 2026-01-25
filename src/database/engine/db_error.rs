use crate::database::domain::DomainError;

#[derive(Debug)]
pub enum EngineError {
    NoTableSelected,
    TableNotFound(String),
    Domain(DomainError),
}
impl From<DomainError> for EngineError {
    fn from(err: DomainError) -> Self {
        EngineError::Domain(err)
    }
}

// impl From<fn(String)> for EngineError {
//     fn from(value: fn(String)) -> Self {
//         value
//     }
// }
