#[derive(Debug)]
pub enum StorageError {
    NotLoaded,
    DatabaseAlreadyExists,
    DatabaseNotFound,

    Io(std::io::Error),
    Serde(serde_json::Error),
    Bincode(bincode::Error),
}

impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Serde(e)
    }
}

impl From<bincode::Error> for StorageError {
    fn from(e: bincode::Error) -> Self {
        StorageError::Bincode(e)
    }
}
