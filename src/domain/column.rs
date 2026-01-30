use serde::{Deserialize, Serialize};

use crate::domain::{DataType, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constraint {
    Nullable,
    NotNull,
    Unique,
    Increment,
    Default(Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    name: String,
    data_type: DataType,
    constraint: Vec<Constraint>,
}

impl Column {
    pub(super) fn new(
        name: impl Into<String>,
        data_type: DataType,
        constraint: Vec<Constraint>,
    ) -> Self {
        Self {
            name: name.into(),
            data_type,
            constraint,
        }
    }
    pub(super) fn name(&self) -> &str {
        &self.name
    }
    pub(super) fn data_type(&self) -> &DataType {
        &self.data_type
    }
}

impl Column {
    pub(super) fn has_constraint(&self, predicate: impl Fn(&Constraint) -> bool) -> bool {
        self.constraint.iter().any(predicate)
    }

    pub(super) fn get_constraint<T>(
        &self,
        extractor: impl Fn(&Constraint) -> Option<T>,
    ) -> Option<T> {
        self.constraint.iter().find_map(extractor)
    }
}
