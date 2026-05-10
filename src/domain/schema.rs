use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, CompareOp, Constraint, DataType, DomainError, Expr, ResolvedCompare, ResolvedExpr,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    columns: Vec<Column>,
}

impl Schema {
    pub(super) fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }
    pub(super) fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub(super) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        let mut new_columns = Vec::with_capacity(columns.len());

        let mut seen = HashSet::new();

        for (name, data_type, constraint) in columns {
            if !seen.insert(name) {
                return Err(DomainError::DuplicateColumnName(name.to_string()));
            };

            new_columns.push(Column::new(name, data_type, constraint.to_vec()));
        }

        for column in self.columns() {
            if !seen.insert(column.name()) {
                return Err(DomainError::DuplicateColumnName(column.name().to_string()));
            };
        }

        self.columns.extend(new_columns);
        Ok(())
    }

    pub(super) fn remove_at(&mut self, index: usize) {
        self.columns.remove(index);
    }
}

// VALIDATOR
impl Schema {
    pub(super) fn bind_expr(&self, expr: &Expr) -> Result<ResolvedExpr, DomainError> {
        match expr {
            Expr::Compare(cmp) => {
                let index = self.resolve_column(&cmp.column)?;
                let column = &self.columns[index];

                let value = match cmp.op {
                    CompareOp::IsNull | CompareOp::IsNotNull => None,

                    _ => {
                        let v = cmp.value.clone().ok_or(DomainError::NotAllowedNull)?;

                        Some(column.data_type().coerce_value(v)?)
                    }
                };

                Ok(ResolvedExpr::Compare(ResolvedCompare::new(
                    index,
                    cmp.op.clone(),
                    value,
                )))
            }

            Expr::And(xs) => {
                let items = xs
                    .iter()
                    .map(|x| self.bind_expr(x))
                    .collect::<Result<_, _>>()?;

                Ok(ResolvedExpr::And(items))
            }

            Expr::Or(xs) => {
                let items = xs
                    .iter()
                    .map(|x| self.bind_expr(x))
                    .collect::<Result<_, _>>()?;

                Ok(ResolvedExpr::Or(items))
            }

            Expr::Not(inner) => Ok(ResolvedExpr::Not(Box::new(self.bind_expr(inner)?))),
        }
    }

    /// Validates column existence and returns its index
    pub(super) fn resolve_column(&self, name: &str) -> Result<usize, DomainError> {
        self.columns
            .iter()
            .position(|column| column.name() == name)
            .ok_or(DomainError::ColumnNotFound(name.to_string()))
    }
}
