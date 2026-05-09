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
