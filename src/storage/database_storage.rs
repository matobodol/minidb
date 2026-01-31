use crate::{domain::Database, storage::StorageError};

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
}
