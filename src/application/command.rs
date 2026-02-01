use crate::domain::{Condition, Constraint, DataType, Value};

#[derive(Debug)]
pub enum Command {
    // ===== REPL =====
    Exit,

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
        values: Vec<(String, Value)>,
    },

    UpdateWhere {
        table: String,
        assignments: Vec<(String, Value)>,
        conditions: Vec<Condition>,
    },
    DeleteWhere {
        table: String,
        conditions: Vec<Condition>,
    },

    // ===== SELECT =====
    SelectAll {
        table: String,
    },
    SelectWhere {
        table: String,
        condition: Condition,
    },
    SelectColumns {
        table: String,
        columns: Vec<String>,
    },
    SelectWhereColumns {
        table: String,
        condition: Condition,
        columns: Vec<String>,
    },
}
impl Command {
    pub fn is_exit(&self) -> bool {
        matches!(self, Command::Exit)
    }
}

#[derive(Debug)]
pub struct ColumnDef {
    pub name: String,
    pub data_type: DataType,
    pub constraints: Vec<Constraint>,
}
