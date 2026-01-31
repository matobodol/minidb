use crate::{
    application::{app_error::AppError, map_domain_error, map_storage_error},
    domain::{Column, Database, DomainError},
    storage::DatabaseStorage,
};

#[derive(Debug)]
pub struct AppManager<S: DatabaseStorage> {
    loaded: S,
    current: Option<String>,
}

impl<S: DatabaseStorage> AppManager<S> {
    pub fn new(storage: S) -> Self {
        Self {
            loaded: storage,
            current: None,
        }
    }
}

// STORAGE
impl<S: DatabaseStorage> AppManager<S> {
    fn create(&mut self, name: &str) -> Result<(), AppError> {
        self.loaded.create(name).map_err(map_storage_error)
    }

    fn load(&mut self, name: &str) -> Result<(), AppError> {
        self.loaded.load(name).map_err(map_storage_error)
    }

    fn unload(&mut self, name: &str) -> Result<(), AppError> {
        self.loaded.unload(&name).map_err(map_storage_error)
        // self.loaded.unload(name).map_err(map_storage_error)
    }

    fn get(&self, name: &str) -> Option<&Database> {
        self.loaded.get(name)
    }

    fn get_mut(&mut self, name: &str) -> Option<&mut Database> {
        self.loaded.get_mut(name)
    }
    fn list(&self) -> Vec<String> {
        self.loaded.list()
    }

    fn exists(&self, name: &str) -> bool {
        self.loaded.exists(name)
    }
}

// LIFECYVLE
impl<S: DatabaseStorage> AppManager<S> {
    fn unload_current(&mut self) -> Result<(), AppError> {
        if let Some(name) = self.current.take() {
            self.unload(&name)?;
        }
        Ok(())
    }

    pub fn create_database(&mut self, name: &str) -> Result<(), AppError> {
        if self.exists(name) {
            return Err(AppError::DatabaseAlreadyExists);
        }

        self.create(name)
    }

    pub fn use_database(&mut self, name: &str) -> Result<(), AppError> {
        if !self.exists(name) {
            return Err(AppError::DatabaseNotFound);
        }

        if self.current.as_deref() == Some(name) {
            return Ok(());
        }

        self.unload_current()?;
        self.load(name)?;
        self.current = Some(name.to_string());

        Ok(())
    }

    pub fn drop_database(&mut self, name: &str) -> Result<(), AppError> {
        if !self.exists(name) {
            return Err(AppError::DatabaseNotFound);
        }

        if self.current.as_deref() == Some(name) {
            return Err(AppError::InvalidOperation(
                "cannot drop currently used database".into(),
            ));
        }

        self.loaded.drop(name).map_err(map_storage_error)
    }

    pub fn show_current(&self) -> Result<String, AppError> {
        self.current.clone().ok_or(AppError::NoDatabaseSelected)
    }

    pub fn show_databases(&self) -> Vec<String> {
        self.list()
    }
}

// HELPER INTERNAL
impl<S: DatabaseStorage> AppManager<S> {
    // mutable
    fn with_db_mut<T>(
        &mut self,
        f: impl FnOnce(&mut Database) -> Result<T, DomainError>,
    ) -> Result<T, AppError> {
        let name = self.current.clone().ok_or(AppError::NoDatabaseSelected)?;

        let db = self
            // .loaded
            .get_mut(&name)
            .ok_or(AppError::DatabaseNotFound)?;

        // 1. jalankan mutasi domain
        let result = f(db).map_err(map_domain_error)?;

        // 2. SAVE JIKA SUKSES
        self.loaded.save(&name).map_err(map_storage_error)?;

        Ok(result)
    }

    // read only
    fn with_db<T>(
        &self,
        f: impl FnOnce(&Database) -> Result<T, crate::domain::DomainError>,
    ) -> Result<T, AppError> {
        let name = self.current.as_ref().ok_or(AppError::NoDatabaseSelected)?;

        let db = self.get(name).ok_or(AppError::DatabaseNotFound)?;

        f(db).map_err(map_domain_error)
    }
}

// TABLE OPERATION
impl<S: DatabaseStorage> AppManager<S> {
    pub fn create_table(&mut self, name: &str) -> Result<(), AppError> {
        // self.with_db_mut(|db| db.create_table(name))
        self.with_db_mut(|db| db.create_table(name))
    }

    pub fn drop_table(&mut self, name: &str) -> Result<usize, AppError> {
        self.with_db_mut(|db| db.drop_table(name))
    }

    pub fn show_tables(&self) -> Result<Vec<String>, AppError> {
        self.with_db(|db| Ok(db.list_tables()))
    }

    pub fn describe_table(&self, table: &str) -> Result<Vec<Column>, AppError> {
        self.with_db(|db| db.describe_table(table))
    }
}

// COLUMN OPERATION
use crate::domain::{Constraint, DataType};

impl<S: DatabaseStorage> AppManager<S> {
    pub fn add_columns(
        &mut self,
        table: &str,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), AppError> {
        self.with_db_mut(|db| db.add_columns(table, columns))
    }

    pub fn delete_columns(&mut self, table: &str, columns: Vec<String>) -> Result<usize, AppError> {
        self.with_db_mut(|db| db.delete_column(table, columns))
    }
}

// ROW OPERATION
use crate::domain::{Condition, Value};

impl<S: DatabaseStorage> AppManager<S> {
    pub fn insert_row(&mut self, table: &str, values: &[(&str, Value)]) -> Result<(), AppError> {
        self.with_db_mut(|db| db.insert_row(table, values))
    }

    pub(crate) fn update_where(
        &mut self,
        table: &str,
        conditions: &[Condition],
        assignments: &[(String, Value)],
    ) -> Result<usize, AppError> {
        self.with_db_mut(|db| db.update_where(table, conditions, assignments))
    }
    pub(crate) fn delete_where(
        &mut self,
        table: &str,
        conditions: &[Condition],
    ) -> Result<usize, AppError> {
        self.with_db_mut(|db| db.delete_where(table, conditions))
    }
}

// SELECT (Read-Only-Api)
impl<S: DatabaseStorage> AppManager<S> {
    pub fn select_all(&self, table: &str) -> Result<Vec<Vec<Value>>, AppError> {
        self.with_db(|db| db.select_all(table))
    }

    pub fn select_where(
        &self,
        table: &str,
        condition: Condition,
    ) -> Result<Vec<Vec<Value>>, AppError> {
        self.with_db(|db| db.select_where(table, condition))
    }

    pub fn select_columns(
        &self,
        table: &str,
        columns: &[&str],
    ) -> Result<Vec<Vec<Value>>, AppError> {
        self.with_db(|db| db.select_columns(table, columns))
    }

    pub fn select_where_columns(
        &self,
        table: &str,
        condition: Condition,
        columns: &[&str],
    ) -> Result<Vec<Vec<Value>>, AppError> {
        self.with_db(|db| db.select_where_columns(table, condition, columns))
    }
}

// impl AppManager {
//     pub fn new() -> Self {
//         Self {
//             loaded: HashMap::new(),
//             current: None,
//         }
//     }
//     pub fn create(&mut self, name: &str) -> Result<(), AppError> {
//         if self.loaded.contains_key(name) {
//             return Err(AppError::DatabaseAlreadyExists);
//         }
//         self.loaded.insert(name.to_string(), Database::new());
//
//         Ok(())
//     }
//
//     pub fn db_use(&mut self, name: &str) {
//         self.current = Some(name.to_string());
//     }
//
//     fn db_mut(&mut self) -> Result<&mut Database, AppError> {
//         let name = self.current.as_ref().ok_or(AppError::NoDatabaseSelected)?;
//
//         self.loaded.get_mut(name).ok_or(AppError::DatabaseNotFound)
//     }
//
//     fn db(&self) -> Result<&Database, AppError> {
//         let name = self.current.as_ref().ok_or(AppError::NoDatabaseSelected)?;
//
//         self.loaded.get(name).ok_or(AppError::DatabaseNotFound)
//     }
// }
