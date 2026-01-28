#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Str,
    Float,
    // Date,
    Enum { variants: Vec<String> },
}
impl DataType {
    pub fn matches(&self, value: &Value) -> bool {
        match (self, value) {
            (DataType::Int, Value::Int(_)) => true,
            (DataType::Str, Value::Str(_)) => true,
            (DataType::Float, Value::Float(_)) => true,
            (DataType::Enum { variants: allowed }, Value::Enum { value: val }) => {
                allowed.contains(val)
            }
            (_, Value::Null) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Str(String),
    Float(f64),
    // Date(chrono::NaiveDate),
    Enum { value: String },
    Null,
}
impl Value {
    pub fn compare(&self, op: &Cmp, to_cmp: &Value) -> bool {
        match (op, self, to_cmp) {
            (Cmp::Eq, Value::Null, _) => true,
            (Cmp::Eq, Value::Int(a), Value::Int(b)) => a == b,
            (Cmp::Eq, Value::Str(a), Value::Str(b)) => a == b,
            (Cmp::Eq, Value::Float(a), Value::Float(b)) => a == b,
            (Cmp::Eq, Value::Enum { value: a }, Value::Enum { value: b }) => a == b,
            (Cmp::Gt, Value::Int(a), Value::Int(b)) => a > b,
            (Cmp::Lt, Value::Int(a), Value::Int(b)) => a < b,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedCondition {
    pub index: usize,
    pub cmp: Cmp,
    pub value: Value,
}
#[derive(Debug, Clone)]
pub struct Condition {
    pub column: String,
    pub cmp: Cmp,
    pub value: Value,
}
impl Condition {
    pub fn eq(column: &str, value: Value) -> Self {
        Self {
            column: column.to_string(),
            cmp: Cmp::Eq,
            value,
        }
    }

    pub fn lt(column: &str, value: Value) -> Self {
        Self {
            column: column.to_string(),
            cmp: Cmp::Lt,
            value,
        }
    }

    pub fn gt(column: &str, value: Value) -> Self {
        Self {
            column: column.to_string(),
            cmp: Cmp::Gt,
            value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cmp {
    Eq,
    Lt,
    Gt,
}

#[derive(Debug)]
pub enum DomainError {
    InternalError,
    NotAllowedNull,
    ValueNotFound {
        miss_value: Value,
        in_the_column: String,
        reason: String,
    },
    ColumnNotFound,
    DuplicateColumnName,
    DuplicateTableName,
    TableNotFound(String),
    ColumnCountMismatch {
        expected: usize,
        found: usize,
    },

    TypeMismatch {
        column_index: usize,
        expected: DataType,
        found: Value,
    },
}
