use std::collections::HashMap;

use prettytable::{Cell, Row as ROW, Table as PT, format};

use crate::{
    application::{app_error::AppError, map_domain_error, map_storage_error},
    domain::{Column, Condition, DataType, Database, DomainError},
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

impl<S: DatabaseStorage> AppManager<S> {
    // mutable
    pub fn with_db_mut<T>(
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
    pub fn with_db<T>(
        &self,
        f: impl FnOnce(&Database) -> Result<T, DomainError>,
    ) -> Result<T, AppError> {
        let name = self.current.as_ref().ok_or(AppError::NoDatabaseSelected)?;

        let db = self.get(name).ok_or(AppError::DatabaseNotFound)?;

        f(db).map_err(map_domain_error)
    }
}
// Lookup API for application layer (read-only)
impl<S: DatabaseStorage> AppManager<S> {
    pub fn select_all(&self, table: &str) -> Result<(), AppError> {
        let rows = self.with_db(|tbl| tbl.select_all(table))?;
        let columns = self.with_db(|tbl| tbl.columns(table))?;
        print_select(columns, rows);
        Ok(())
    }

    pub fn select_where(&self, table: &str, condition: Condition) -> Result<(), AppError> {
        let rows = self.with_db(|tbl| tbl.select_where(table, condition))?;
        let columns = self.with_db(|tbl| tbl.columns(table))?;

        print_select(columns, rows);

        Ok(())
    }

    pub fn select_columns(&self, table: &str, columns: &[&str]) -> Result<(), AppError> {
        let rows = self.with_db(|tbl| tbl.select_columns(table, columns))?;
        print_select_column(columns, rows);

        Ok(())
    }

    pub fn select_where_columns(
        &self,
        table: &str,
        condition: Condition,
        columns: &[&str],
    ) -> Result<(), AppError> {
        let rows = self.with_db(|tbl| tbl.select_where_columns(table, condition, columns))?;
        print_select_column(columns, rows);

        Ok(())
    }
}

pub fn print_select(columns: Vec<Column>, rows: Vec<Vec<String>>) {
    let mut pt = PT::new();
    pt.set_format(*format::consts::FORMAT_BOX_CHARS);

    let mut format = HashMap::new();

    let cells = columns
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let align = match c.data_type() {
                DataType::Int => "r",
                DataType::Float => "r",
                DataType::Str => "l",
                DataType::Enum { variants: _ } => "c",
            };
            format.insert(i, align);

            if c.has_constraint(|c| matches!(c, crate::domain::Constraint::Unique)) {
                let name = format!("*{}", c.name());
                Cell::new(&name)
                    .style_spec("c")
                    .with_style(prettytable::Attr::Bold)
            } else {
                Cell::new(c.name())
                    .style_spec("c")
                    .with_style(prettytable::Attr::Bold)
            }
        })
        .collect();

    pt.add_row(ROW::new(cells));

    rows.into_iter().for_each(|row| {
        let cells = row
            .iter()
            .enumerate()
            .map(|(i, value)| {
                if let Some(align) = format.get(&i) {
                    if value == "-" {
                        Cell::new(value).style_spec("c")
                    } else {
                        Cell::new(value).style_spec(align)
                    }
                } else {
                    Cell::new("-").style_spec("c")
                }
            })
            .collect();
        pt.add_row(ROW::new(cells));
    });

    pt.printstd();
}

pub fn print_select_column(columns: &[&str], rows: Vec<Vec<String>>) {
    let mut pt = PT::new();
    pt.set_format(*format::consts::FORMAT_BOX_CHARS);

    let cells = columns
        .iter()
        .map(|c| Cell::new(c).style_spec("c"))
        .collect();

    pt.add_row(ROW::new(cells));

    for row in rows {
        let cells = row
            .iter()
            .map(|value| Cell::new(value).style_spec("r"))
            .collect();
        pt.add_row(ROW::new(cells));
    }

    pt.printstd();
}
