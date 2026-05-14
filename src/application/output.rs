use std::time::Instant;

pub struct QueryTimer {
    start: Instant,
}

impl QueryTimer {
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
}

#[derive(Debug)]
pub enum QueryInfo {
    Silent,
    Select { rows: usize, elapsed_ms: f64 },
    Insert { affected: usize, elapsed_ms: f64 },
    Update { affected: usize, elapsed_ms: f64 },
    Delete { affected: usize, elapsed_ms: f64 },
    Describe { columns: usize, elapsed_ms: f64 },

    CreateDatabase { name: String, elapsed_ms: f64 },
    DropDatabase { name: String, elapsed_ms: f64 },
    UseDatabase { name: String, elapsed_ms: f64 },

    CreateTable { name: String, elapsed_ms: f64 },
    DropTable { name: String, elapsed_ms: f64 },

    AlterAddColumn { affected: usize, elapsed_ms: f64 },
    AlterDropColumn { affected: usize, elapsed_ms: f64 },

    ShowTables { count: usize, elapsed_ms: f64 },
    ShowDatabases { count: usize, elapsed_ms: f64 },

    Exit,
}

pub fn print_query_error(err: impl std::fmt::Display) {
    println!("ERROR: {err}");
}

fn suffix_time(elapsed_ms: f64) -> String {
    format!(" | time: {:.3} ms", elapsed_ms)
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

pub fn print_query_result(info: QueryInfo) {
    match info {
        // =====================
        // SELECT
        // =====================
        QueryInfo::Select { rows, elapsed_ms } => match rows {
            0 => println!("Empty set{}", suffix_time(elapsed_ms)),
            1 => println!("(1 row){}", suffix_time(elapsed_ms)),
            n => println!("({n} rows){}", suffix_time(elapsed_ms)),
        },

        // =====================
        // ROW MUTATION
        // =====================
        QueryInfo::Insert {
            affected,
            elapsed_ms,
        } => {
            println!(
                "{} row{} inserted{}",
                affected,
                plural(affected),
                suffix_time(elapsed_ms),
            );
        }

        QueryInfo::Update {
            affected,
            elapsed_ms,
        } => {
            println!(
                "{} row{} affected{}",
                affected,
                plural(affected),
                suffix_time(elapsed_ms),
            );
        }

        QueryInfo::Delete {
            affected,
            elapsed_ms,
        } => {
            println!(
                "{} row{} deleted{}",
                affected,
                plural(affected),
                suffix_time(elapsed_ms),
            );
        }

        // =====================
        // DATABASE
        // =====================
        QueryInfo::CreateDatabase { name, .. } => {
            println!("Database '{name}' created");
        }

        QueryInfo::DropDatabase { name, .. } => {
            println!("Database '{name}' dropped");
        }

        QueryInfo::UseDatabase { name, .. } => {
            println!("Using database '{name}'");
        }

        QueryInfo::ShowDatabases { count, .. } => {
            println!("({} database{})", count, plural(count));
        }

        // =====================
        // TABLE
        // =====================
        QueryInfo::CreateTable { name, .. } => {
            println!("Table '{name}' created");
        }

        QueryInfo::DropTable { name, .. } => {
            println!("Table '{name}' dropped");
        }

        QueryInfo::ShowTables { count, .. } => {
            println!("({} table{})", count, plural(count));
        }

        QueryInfo::Describe {
            columns,
            elapsed_ms,
        } => {
            println!(
                "({columns} column{}){}",
                plural(columns),
                suffix_time(elapsed_ms),
            );
        }

        // =====================
        // ALTER
        // =====================
        QueryInfo::AlterAddColumn { affected, .. } => {
            println!("{} column{} added", affected, plural(affected));
        }

        QueryInfo::AlterDropColumn { affected, .. } => {
            println!("{} column{} dropped", affected, plural(affected));
        }

        // =====================
        // REPL
        // =====================
        QueryInfo::Exit => {
            println!("bye");
        }

        QueryInfo::Silent => {}
    }

    println!()
}
