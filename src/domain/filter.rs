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
    pub(super) fn new(index: usize, op: CompareOp, value: Option<Value>) -> Self {
        Self { index, op, value }
    }
}

pub(super) fn compare(left: &Value, op: &CompareOp, right: &Value) -> bool {
    use std::cmp::Ordering::*;

    match op {
        CompareOp::Eq => eq(left, right),
        CompareOp::Ne => !eq(left, right),

        CompareOp::Lt => ord(left, right, |o| o == Less),
        CompareOp::Gt => ord(left, right, |o| o == Greater),
        CompareOp::Lte => ord(left, right, |o| o != Greater),
        CompareOp::Gte => ord(left, right, |o| o != Less),

        // CompareOp::IsNull => matches!(left, Value::Null),
        // CompareOp::IsNotNull => !matches!(left, Value::Null),
        _ => false,
    }
}

fn eq(left: &Value, right: &Value) -> bool {
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => a == b,
        (Value::Float(a), Value::Float(b)) => a == b,
        (Value::Str(a), Value::Str(b)) => a == b,
        (Value::Enum { value: a }, Value::Enum { value: b }) => a == b,
        _ => false,
    }
}

fn ord<F>(left: &Value, right: &Value, cmp: F) -> bool
where
    F: FnOnce(std::cmp::Ordering) -> bool,
{
    match (left, right) {
        (Value::Int(a), Value::Int(b)) => cmp(a.cmp(b)),
        (Value::Float(a), Value::Float(b)) => a.partial_cmp(b).is_some_and(cmp),
        _ => false,
    }
}
