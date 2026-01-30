use std::collections::HashMap;

use crate::{application::app_error::AppError, domain::Database};

#[derive(Debug)]
pub struct AppManager {
    loaded: HashMap<String, Database>,
    current: Option<String>,
}

impl AppManager {
    pub fn new() -> Self {
        Self {
            loaded: HashMap::new(),
            current: None,
        }
    }
    pub fn create(&mut self, name: &str) -> Result<(), AppError> {
        if self.loaded.contains_key(name) {
            return Err(AppError::DatabaseAlreadyExists);
        }
        self.loaded.insert(name.to_string(), Database::new());

        Ok(())
    }

    pub fn db_use(&mut self, name: &str) {
        self.current = Some(name.to_string());
    }
    // =========================
    // DOMAIN ACCESS
    // =========================

    pub fn db_mut(&mut self) -> Result<&mut Database, AppError> {
        let name = self.current.as_ref().ok_or(AppError::NoDatabaseSelected)?;

        self.loaded.get_mut(name).ok_or(AppError::DatabaseNotFound)
    }

    pub fn db(&self) -> Result<&Database, AppError> {
        let name = self.current.as_ref().ok_or(AppError::NoDatabaseSelected)?;

        self.loaded.get(name).ok_or(AppError::DatabaseNotFound)
    }
}
