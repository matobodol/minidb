use crate::{
    application::{AppError, AppManager, Command},
    domain::{Column, Constraint, DataType, Value},
    storage::DatabaseStorage,
};

#[derive(Debug)]
pub enum CommandOutput {
    Ok,
    Affected(usize),
    Rows(Vec<Vec<Value>>),
    Columns(Vec<Column>),
    Message(String),
    Exit,
}

pub fn execute_command<S: DatabaseStorage>(
    app: &mut AppManager<S>,
    command: Command,
) -> Result<CommandOutput, AppError> {
    match command {
        // ===== DATABASE =====
        Command::CreateDatabase { name } => {
            app.create_database(&name)?;
            Ok(CommandOutput::Ok)
        }

        Command::UseDatabase { name } => {
            app.use_database(&name)?;
            Ok(CommandOutput::Ok)
        }

        Command::DropDatabase { name } => {
            app.drop_database(&name)?;
            Ok(CommandOutput::Ok)
        }

        Command::ShowCurrentDatabase => {
            let name = app.show_current()?;
            Ok(CommandOutput::Message(name))
        }

        Command::ShowDatabases => {
            let dbs = app.show_databases();
            Ok(CommandOutput::Message(dbs.join("\n")))
        }

        // ===== TABLE =====
        Command::CreateTable { name } => {
            app.create_table(&name)?;
            Ok(CommandOutput::Ok)
        }

        Command::DropTable { name } => {
            let affected = app.drop_table(&name)?;
            Ok(CommandOutput::Affected(affected))
        }

        Command::ShowTables => {
            let tables = app.show_tables()?;
            Ok(CommandOutput::Message(tables.join("\n")))
        }

        Command::DescribeTable { table } => {
            let columns = app.describe_table(&table)?;

            Ok(CommandOutput::Columns(columns))
        }

        // ===== COLUMN =====
        Command::AlterTableAddColumn { table, columns } => {
            let cols: Vec<(&str, DataType, &[Constraint])> = columns
                .iter()
                .map(|(name, ty, cons)| (name.as_str(), ty.clone(), cons.as_slice()))
                .collect();

            app.add_columns(&table, cols)?;
            Ok(CommandOutput::Ok)
        }
        Command::AlterTableDropColumn { table, columns } => {
            let affected = app.delete_columns(&table, columns)?;
            Ok(CommandOutput::Affected(affected))
        }

        // ===== ROW =====
        Command::InsertRow { table, values } => {
            let vals: Vec<(&str, Value)> = values
                .iter()
                .map(|(k, v)| (k.as_str(), v.clone()))
                .collect();

            app.insert_row(&table, &vals)?;
            Ok(CommandOutput::Ok)
        }

        Command::UpdateWhere {
            table,
            assignments,
            conditions,
        } => {
            let count = app.update_where(&table, &conditions, &assignments)?;
            Ok(CommandOutput::Affected(count))
        }

        Command::DeleteWhere { table, conditions } => {
            let count = app.delete_where(&table, &conditions)?;
            Ok(CommandOutput::Affected(count))
        }

        // ===== SELECT =====
        Command::SelectAll { table } => {
            let rows = app.select_all(&table)?;
            Ok(CommandOutput::Rows(rows))
        }

        Command::SelectWhere { table, condition } => {
            let rows = app.select_where(&table, condition)?;
            Ok(CommandOutput::Rows(rows))
        }

        Command::SelectColumns { table, columns } => {
            let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            let rows = app.select_columns(&table, &cols)?;
            Ok(CommandOutput::Rows(rows))
        }

        Command::SelectWhereColumns {
            table,
            condition,
            columns,
        } => {
            let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            let rows = app.select_where_columns(&table, condition, &cols)?;
            Ok(CommandOutput::Rows(rows))
        }

        // ===== META =====
        Command::Exit => Ok(CommandOutput::Exit),
    }
}
