use crate::{domain::Database, storage::StorageError};

use std::path::{Path, PathBuf};

/// Default root directory for MiniDB data
pub fn default_root_path() -> PathBuf {
    // Allow override via environment variable
    if let Ok(custom_path) = std::env::var("MINIDB_HOME") {
        let path = PathBuf::from(custom_path).join("storage");
        std::fs::create_dir_all(&path).ok();
        return path;
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".minidb").join("storage")
}

pub trait DatabaseStorage {
    fn create(&mut self, name: &str) -> Result<(), StorageError>;
    fn drop(&mut self, name: &str) -> Result<(), StorageError>;

    fn load(&mut self, name: &str) -> Result<(), StorageError>;
    fn unload(&mut self, name: &str) -> Result<(), StorageError>;

    fn get(&self, name: &str) -> Option<&Database>;
    fn get_mut(&mut self, name: &str) -> Option<&mut Database>;

    fn exists(&self, name: &str) -> bool;
    fn list(&self) -> Vec<String>;
    fn save(&mut self, name: &str) -> Result<(), StorageError>;

    /// Get the root path where databases are stored
    fn root_path(&self) -> &Path;
}
