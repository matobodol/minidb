use std::{collections::HashMap, fs, path::PathBuf};

use crate::{
    domain::Database,
    storage::{DatabaseStorage as Storage, StorageError, default_root_path},
};

#[derive(Debug)]
pub struct BincodeStorage {
    root: PathBuf,
    loaded: HashMap<String, Database>,
}

impl BincodeStorage {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        std::fs::create_dir_all(&path).ok();

        Self {
            root: path,
            loaded: HashMap::new(),
        }
    }

    /// Create storage with default path (~/.minidb/storage)
    pub fn with_default() -> Self {
        let path = default_root_path();
        Self::new(path)
    }

    fn db_path(&self, name: &str) -> PathBuf {
        self.root.join(format!("{name}.bin"))
    }
}

impl Storage for BincodeStorage {
    fn root_path(&self) -> &std::path::Path {
        &self.root
    }

    fn save(&mut self, name: &str) -> Result<(), StorageError> {
        let db = self.loaded.get(name).ok_or(StorageError::NotLoaded)?;
        let path = self.db_path(name);

        let bytes = bincode::serialize(db)?;
        fs::write(path, bytes)?;

        Ok(())
    }

    fn create(&mut self, name: &str) -> Result<(), StorageError> {
        let path = self.db_path(name);

        if path.exists() {
            return Err(StorageError::DatabaseAlreadyExists);
        }

        let db = Database::new();

        let bytes = bincode::serialize(&db)?;
        fs::write(&path, bytes)?;

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

        fs::remove_file(path)?;

        Ok(())
    }

    fn load(&mut self, name: &str) -> Result<(), StorageError> {
        if self.loaded.contains_key(name) {
            return Ok(());
        }

        let path = self.db_path(name);
        let bytes = fs::read(&path)?;
        let mut db: Database = bincode::deserialize(&bytes)?;

        // REBUILD INDEX AFTER DESERIALIZATION
        db.rebuild_indices();

        self.loaded.insert(name.to_string(), db);
        Ok(())
    }

    fn unload(&mut self, name: &str) -> Result<(), StorageError> {
        let db = self.loaded.remove(name).ok_or(StorageError::NotLoaded)?;

        let path = self.db_path(name);

        let bytes = bincode::serialize(&db)?;
        fs::write(&path, bytes)?;

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
                let path = e.path();

                if path.extension()? != "bin" {
                    return None;
                }

                path.file_stem().and_then(|s| s.to_str()).map(String::from)
            })
            .collect()
    }
}
