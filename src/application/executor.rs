use crate::{
    application::{AppError, AppManager, Command, QueryInfo, QueryTimer, help},
    domain::{Constraint, DataType},
    storage::DatabaseStorage,
};

// pub fn execute<S: DatabaseStorage>(cmd: Command, app: &mut AppManager<S>) -> Result<(), AppError> {
pub fn execute<S: DatabaseStorage>(
    cmd: Command,
    app: &mut AppManager<S>,
) -> Result<QueryInfo, AppError> {
    match cmd {
        // ===== REPL =====
        Command::Exit => Ok(QueryInfo::Exit),
        Command::Help => {
            help();
            Ok(QueryInfo::Silent)
        }
        Command::DebugDatabase => {
            app.with_db(|db| db.debug())?;
            Ok(QueryInfo::Silent)
        }
        Command::DebugTable { name } => {
            app.with_db(|db| db.debug_table(&name))?;
            Ok(QueryInfo::Silent)
        }

        // ===== DATABASE =====
        Command::CreateDatabase { name } => {
            let timer = QueryTimer::start();
            app.create_database(&name)?;
            Ok(QueryInfo::CreateDatabase {
                name,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::UseDatabase { name } => {
            let timer = QueryTimer::start();
            app.use_database(&name)?;
            Ok(QueryInfo::UseDatabase {
                name,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::DropDatabase { name } => {
            let timer = QueryTimer::start();
            app.drop_database(&name)?;
            Ok(QueryInfo::DropDatabase {
                name,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::ShowDatabases => {
            let timer = QueryTimer::start();
            let databases = app.show_databases();
            for db in &databases {
                println!("  - {db}");
            }
            Ok(QueryInfo::ShowDatabases {
                count: databases.len(),
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::ShowCurrentDatabase => {
            println!("{}", app.show_current_database().unwrap_or("none".into()));
            Ok(QueryInfo::Silent)
        }

        // ===== TABLE =====
        Command::CreateTable { name } => {
            let timer = QueryTimer::start();
            app.with_db_mut(|db| db.create_table(&name))?;
            Ok(QueryInfo::CreateTable {
                name,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::DropTable { name } => {
            let timer = QueryTimer::start();
            app.with_db_mut(|db| db.drop_table(&name))?;
            Ok(QueryInfo::DropTable {
                name,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::ShowTables => {
            let timer = QueryTimer::start();
            let tables = app.with_db(|db| Ok(db.list_tables()))?;
            for t in &tables {
                println!("  - {t}");
            }
            Ok(QueryInfo::ShowTables {
                count: tables.len(),
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::DescribeTable { table } => {
            let timer = QueryTimer::start();
            let columns = app.describe(&table)?;
            Ok(QueryInfo::Describe {
                columns,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        // ===== COLUMN =====
        Command::AlterTableAddColumn { table, columns } => {
            let timer = QueryTimer::start();
            let affected = columns.len();
            let cols: Vec<(&str, DataType, &[Constraint])> = columns
                .iter()
                .map(|(n, dt, c)| (n.as_str(), dt.clone(), c.as_slice()))
                .collect();
            app.with_db_mut(|db| db.add_columns(&table, cols))?;
            Ok(QueryInfo::AlterAddColumn {
                affected,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        Command::AlterTableDropColumn { table, columns } => {
            let timer = QueryTimer::start();
            let affected = columns.len();
            app.with_db_mut(|db| db.delete_columns(&table, columns))?;
            Ok(QueryInfo::AlterDropColumn {
                affected,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        // ===== Row =====
        Command::InsertRow {
            table,
            columns,
            rows,
        } => {
            let timer = QueryTimer::start();
            let affected = app.with_db_mut(|db| db.insert(&table, columns, rows))?;

            Ok(QueryInfo::Insert {
                affected,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        Command::UpdateWhere {
            table,
            assignments,
            conditions,
        } => {
            let timer = QueryTimer::start();
            let updated = app.with_db_mut(|db| db.update_rows(&table, assignments, &conditions))?;

            Ok(QueryInfo::Update {
                affected: updated,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        Command::Delete { table, conditions } => {
            let timer = QueryTimer::start();
            let deleted = app.with_db_mut(|db| db.delete_rows(&table, &conditions))?;

            Ok(QueryInfo::Delete {
                affected: deleted,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        // ===== SELECT =====
        Command::SelectAll { table } => {
            let timer = QueryTimer::start();
            let rows = app.lookup_all(&table)?;

            Ok(QueryInfo::Select {
                rows,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::SelectColumns { table, columns } => {
            let timer = QueryTimer::start();
            let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            let rows = app.lookup_columns(&table, &cols)?;

            Ok(QueryInfo::Select {
                rows,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
        Command::SelectWhere { table, conditions } => {
            let timer = QueryTimer::start();
            let rows = app.lookup_where(&table, &conditions)?;

            Ok(QueryInfo::Select {
                rows,
                elapsed_ms: timer.elapsed_ms(),
            })
        }

        Command::SelectColumnsWhere {
            table,
            columns,
            conditions,
        } => {
            let timer = QueryTimer::start();
            let cols: Vec<&str> = columns.iter().map(|s| s.as_str()).collect();
            let rows = app.lookup_columns_where(&table, &conditions, &cols)?;

            Ok(QueryInfo::Select {
                rows,
                elapsed_ms: timer.elapsed_ms(),
            })
        }
    }
}
