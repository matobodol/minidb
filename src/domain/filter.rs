use crate::domain::Value;

#[derive(Debug, Clone)]
pub enum Expr {
    Compare(CompareExpr),

    And(Vec<Expr>),
    Or(Vec<Expr>),

    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum ResolvedExpr {
    Compare(ResolvedCompare),

    And(Vec<ResolvedExpr>),
    Or(Vec<ResolvedExpr>),

    Not(Box<ResolvedExpr>),
}

#[derive(Debug, Clone)]
pub enum CompareOp {
    Eq,
    Ne,
    Lt,
    Lte,
    Gt,
    Gte,

    IsNull,
    IsNotNull,

    In,
    Like,
}

#[derive(Debug, Clone)]
pub struct CompareExpr {
    pub column: String,
    pub op: CompareOp,
    pub value: Option<Value>,
}
impl CompareExpr {
    pub fn new(column: String, op: CompareOp, value: Option<Value>) -> Self {
        Self { column, op, value }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedCompare {
    pub index: usize,
    pub op: CompareOp,
    pub value: Option<Value>,
}
impl ResolvedCompare {
    pub fn new(index: usize, op: CompareOp, value: Option<Value>) -> Self {
        Self { index, op, value }
    }
}

pub struct Compare;
impl Compare {
    pub fn compare(left: &Value, op: &CompareOp, right: &Value) -> bool {
        // if matches!(self, Value::Null) || matches!(other, Value::Null) {
        //     return false;
        // }

        match op {
            CompareOp::Eq => Compare::eq(left, right),
            CompareOp::Ne => !Compare::eq(left, right),
            CompareOp::Lt => Compare::lt(left, right),
            CompareOp::Gt => Compare::gt(left, right),
            CompareOp::Lte => Compare::le(left, right),
            CompareOp::Gte => Compare::ge(left, right),
            CompareOp::IsNull => matches!(right, Value::Null),
            CompareOp::IsNotNull => !matches!(right, Value::Null),
            _ => false,
        }
    }

    fn eq(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::Enum { value: a }, Value::Enum { value: b }) => a == b,
            _ => false,
        }
    }

    fn lt(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a < b,
            (Value::Float(a), Value::Float(b)) => a < b,
            _ => false,
        }
    }

    fn gt(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a > b,
            (Value::Float(a), Value::Float(b)) => a > b,
            _ => false,
        }
    }

    fn le(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a <= b,
            (Value::Float(a), Value::Float(b)) => a <= b,
            _ => false,
        }
    }

    fn ge(left: &Value, right: &Value) -> bool {
        match (left, right) {
            (Value::Int(a), Value::Int(b)) => a >= b,
            (Value::Float(a), Value::Float(b)) => a >= b,
            _ => false,
        }
    }
}
