use crate::{
    application::{AppError, AppManager, Command},
    domain::{Constraint, DataType},
    storage::DatabaseStorage,
};

pub fn execute<S: DatabaseStorage>(cmd: Command, app: &mut AppManager<S>) -> Result<(), AppError> {
    match cmd {
        // ===== REPL =====
        Command::Exit => Ok(()),
        Command::Help => Ok(()),

        // ===== DATABASE =====
        Command::CreateDatabase { name } => app.create_database(&name),
        Command::UseDatabase { name } => app.use_database(&name),
        Command::DropDatabase { name } => app.drop_database(&name),
        Command::ShowDatabases => {
            for db in app.show_databases() {
                println!("{}", db);
            }
            Ok(())
        }
        Command::ShowCurrentDatabase => {
            println!("{}", app.show_current_database().unwrap_or("none".into()));
            Ok(())
        }

        // ===== TABLE =====
        Command::CreateTable { name } => app.with_db_mut(|db| db.create_table(&name)),
        Command::DropTable { name } => {
            app.with_db_mut(|db| db.drop_table(&name))?;
            Ok(())
        }
        Command::ShowTables => {
            let tables = app.with_db(|db| Ok(db.list_tables()))?;
            for t in tables {
                println!("{}", t);
            }
            Ok(())
        }
        Command::DescribeTable { table } => {
            app.describe(&table)?;

            Ok(())
        }

        // ===== COLUMN =====
        Command::AlterTableAddColumn { table, columns } => {
            let cols: Vec<(&str, DataType, &[Constraint])> = columns
                .iter()
                .map(|(n, dt, c)| (n.as_str(), dt.clone(), c.as_slice()))
                .collect();

            app.with_db_mut(|db| db.add_columns(&table, cols))?;
            Ok(())
        }

        Command::AlterTableDropColumn { table, columns } => {
            app.with_db_mut(|db| db.delete_column(&table, columns))?;
            Ok(())
        }

        // ===== Row =====
        Command::InsertRow {
            table,
            columns,
            rows,
        } => {
            app.with_db_mut(|db| db.insert(&table, columns, rows))?;
            Ok(())
        }
        Command::UpdateWhere {
            table,
            assignments,
            conditions,
        } => app.with_db_mut(|db| {
            let updated = db.update_rows(&table, assignments, conditions)?;
            println!("{} rows updated", updated);
            Ok(())
        }),
        Command::Delete { table, conditions } => app.with_db_mut(|db| {
            let deleted = db.delete_rows(&table, &conditions)?;
            println!("{} rows deleted", deleted);
            Ok(())
        }),

        // ===== SELECT =====
        Command::SelectAll { table } => app.select_all(&table),
        Command::SelectWhere { table, conditions } => app.select_where(&table, conditions),
        Command::SelectColumns { table, columns } => {
            let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            app.select_columns(&table, &cols)
        }
        Command::SelectColumnsWhere {
            table,
            columns,
            conditions,
        } => {
            let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            app.select_where_columns(&table, conditions, &cols)
        } // ===== BELUM IMPLEMENT =====
          // _ => Err(AppError::InvalidCommand("not implemented".into())),
    }
}
