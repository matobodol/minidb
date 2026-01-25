#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    Int,
    Str,
    Float,
    // Date,
    Enum { variants: Vec<String> },
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
    pub fn matches(&self, data_type: &DataType) -> bool {
        match (self, data_type) {
            (Value::Int(_), DataType::Int) => true,
            (Value::Str(_), DataType::Str) => true,
            (Value::Float(_), DataType::Float) => true,
            (Value::Enum { value }, DataType::Enum { variants: allowed }) => {
                allowed.contains(value)
            }
            (Value::Null, _) => true,
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum DomainError {
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
