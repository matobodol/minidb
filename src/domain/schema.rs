use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::domain::{
    Column, CompareOp, Constraint, DataType, DomainError, Expr, ResolvedCompare, ResolvedExpr,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Schema {
    columns: Vec<Column>,
    index: HashMap<String, usize>,
}

impl Schema {
    pub(super) fn new() -> Self {
        Self {
            columns: Vec::new(),
            index: HashMap::new(),
        }
    }

    pub(super) fn columns(&self) -> &[Column] {
        &self.columns
    }

    pub(super) fn add_column(
        &mut self,
        columns: Vec<(&str, DataType, &[Constraint])>,
    ) -> Result<(), DomainError> {
        // detect duplicate existing + incoming
        let mut seen: HashSet<&str> = self.columns.iter().map(|c| c.name()).collect();

        let mut new_columns = Vec::with_capacity(columns.len());

        for (name, data_type, constraints) in columns {
            if !seen.insert(name) {
                return Err(DomainError::DuplicateColumnName(name.to_string()));
            }

            new_columns.push(Column::new(name, data_type, constraints.to_vec()));
        }

        // append + update index
        for column in new_columns {
            let idx = self.columns.len();

            self.index.insert(column.name().to_string(), idx);

            self.columns.push(column);
        }

        Ok(())
    }

    pub(super) fn remove_many(&mut self, indexes: &[usize]) {
        for &i in indexes.iter().rev() {
            self.columns.remove(i);
        }

        self.index.clear();

        for (i, col) in self.columns.iter().enumerate() {
            self.index.insert(col.name().to_string(), i);
        }
    }
}

//
// VALIDATOR
//
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
                    index, cmp.op, value,
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

    pub(super) fn resolve_column(&self, name: &str) -> Result<usize, DomainError> {
        self.index
            .get(name)
            .copied()
            .ok_or(DomainError::ColumnNotFound(name.to_string()))
    }
}
