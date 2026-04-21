use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    domain::Database,
    storage::{DatabaseStorage as Storage, StorageError},
};

#[derive(Debug)]
pub struct FileStorage {
    root: PathBuf,
    loaded: HashMap<String, Database>,
}

impl FileStorage {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        std::fs::create_dir_all(&root).ok(); // infra concern

        Self {
            root,
            loaded: HashMap::new(),
        }
    }

    fn db_path(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.json"))
    }
}

impl Storage for FileStorage {
    fn save(&mut self, name: &str) -> Result<(), StorageError> {
        let db = self.loaded.get(name).ok_or(StorageError::NotLoaded)?;
        let path = self.db_path(name);

        let content = serde_json::to_string_pretty(db)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn create(&mut self, name: &str) -> Result<(), StorageError> {
        let path = self.db_path(name);

        if path.exists() {
            return Err(StorageError::DatabaseAlreadyExists);
        }

        let db = Database::new();

        let content = serde_json::to_string_pretty(&db)?;
        fs::write(&path, content)?;

        Ok(())
    }

    fn drop(&mut self, name: &str) -> Result<(), StorageError> {
        if self.loaded.contains_key(name) {
            self.unload(name)?;
        }

        let path = self.db_path(name);

        if !path.exists() {
            return Err(StorageError::DatabaseNotFound);
        }

        std::fs::remove_file(path)?;

        Ok(())
    }

    fn load(&mut self, name: &str) -> Result<(), StorageError> {
        if self.loaded.contains_key(name) {
            return Ok(()); // idempotent
        }

        let path = self.db_path(name);
        let content = fs::read_to_string(&path)?;
        let db: Database = serde_json::from_str(&content)?;

        self.loaded.insert(name.to_string(), db);
        Ok(())
    }

    fn unload(&mut self, name: &str) -> Result<(), StorageError> {
        let db = self.loaded.remove(name).ok_or(StorageError::NotLoaded)?;

        let path = self.db_path(name);
        let content = serde_json::to_string_pretty(&db)?;
        fs::write(&path, content)?;

        Ok(())
    }

    fn get(&self, name: &str) -> Option<&Database> {
        self.loaded.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Database> {
        self.loaded.get_mut(name)
    }

    fn exists(&self, name: &str) -> bool {
        self.db_path(name).exists()
    }

    fn list(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.root) else {
            return Vec::new();
        };

        entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                e.path()
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .map(String::from)
            })
            .collect()
    }
}
