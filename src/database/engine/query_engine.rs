use crate::database::domain::{DataType, Value};

type TableName = String;
type ColumnName = String;

pub enum Operation {
    CreateTable {
        table: TableName,
    },
    DropTable {
        table: TableName,
    },

    AddColumns {
        table: TableName,
        columns: Vec<(ColumnName, DataType)>,
    },
    // DeleteColumn(ColumnName),
    InsertRow {
        table: TableName,
        values: Vec<Value>,
    },
    DeleteRow {
        table: TableName,
        column: ColumnName,
        value: Value,
    },
}
