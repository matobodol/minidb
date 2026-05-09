use crate::domain::{Constraint, DataType, Expr, Value};

#[derive(Debug, Clone)]
pub enum Command {
    // ===== REPL =====
    Exit,
    Help,
    DebugDatabase,
    DebugTable {
        name: String,
    },

    // ===== DATABASE =====
    ShowDatabases,
    ShowCurrentDatabase,
    CreateDatabase {
        name: String,
    },
    UseDatabase {
        name: String,
    },
    DropDatabase {
        name: String,
    },

    // ===== TABLE =====
    CreateTable {
        name: String,
    },
    DropTable {
        name: String,
    },
    ShowTables,
    DescribeTable {
        table: String,
    },

    // ===== COLUMN =====
    AlterTableAddColumn {
        table: String,
        columns: Vec<(String, DataType, Vec<Constraint>)>,
    },

    AlterTableDropColumn {
        table: String,
        columns: Vec<String>,
    },

    // ===== ROW =====
    InsertRow {
        table: String,
        columns: Option<Vec<String>>,
        rows: Vec<Vec<Value>>,
    },

    UpdateWhere {
        table: String,
        assignments: Vec<(String, Value)>,
        conditions: Expr,
    },

    Delete {
        table: String,
        conditions: Expr,
    },

    // ===== SELECT =====
    SelectAll {
        table: String,
    },

    SelectColumns {
        table: String,
        columns: Vec<String>,
    },

    SelectWhere {
        table: String,
        conditions: Expr,
    },

    SelectColumnsWhere {
        table: String,
        columns: Vec<String>,
        conditions: Expr,
    },
}
impl Command {
    pub fn is_exit(&self) -> bool {
        matches!(self, Command::Exit)
    }
    pub fn is_help(&self) -> bool {
        matches!(self, Command::Help)
    }
}
