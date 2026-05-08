use rustyline::DefaultEditor;

use crate::{
    application::{AppManager, executor, parser},
    storage::DatabaseStorage,
};

pub fn print_welcome() {
    println!("╔══════════════════════════════╗");
    println!("║        MINI DB ENGINE        ║");
    println!("║          v0.1.0              ║");
    println!("╚══════════════════════════════╝");
    println!("Type 'exit' to quit");
    println!();
}

pub fn start<S>(app: &mut AppManager<S>)
where
    S: DatabaseStorage,
{
    print_welcome();
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        // PROMPT
        let prompt = if let Ok(db_name) = app.show_current_database() {
            format!("[minidb:{}] > ", db_name)
        } else {
            "minidb > ".to_string()
        };

        let line = rl.readline(&prompt);

        match line {
            Ok(input) => {
                let input = input.trim().trim_end_matches(';');

                if input.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(input);

                // PARSE
                let tokens = tokenize(input);

                // PARSE
                let cmd = match parser::parse(tokens) {
                    Ok(c) => c,
                    Err(e) => {
                        println!("error: {}", e);
                        continue;
                    }
                };

                // EXIT
                if cmd.is_exit() {
                    println!("query ok. Exit..");
                    break;
                }
                if cmd.is_help() {
                    help();
                }

                // EXECUTE
                if let Err(e) = executor::execute(cmd, app) {
                    println!("error: {}", e);
                } else {
                    println!("\nquery ok");
                }
            }

            Err(err) => {
                println!("error: {:?}", err);
                break;
            }
        }
    }
}

pub fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();

    let mut chars = input.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        match c {
            // =====================
            // STRING HANDLING
            // =====================
            '"' => {
                current.push(c);
                in_string = !in_string;

                // jika string ditutup → push token
                if !in_string {
                    tokens.push(current.clone());
                    current.clear();
                }
            }

            // escape \" di dalam string
            '\\' if in_string => {
                current.push(c);

                if let Some(next) = chars.peek() {
                    if *next == '"' {
                        current.push(*next);
                        chars.next();
                    }
                }
            }

            // =====================
            // DELIMITER (luar string)
            // =====================
            '(' | ')' | ',' | '=' if !in_string => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }

                tokens.push(c.to_string());
            }

            // =====================
            // WHITESPACE (luar string)
            // =====================
            c if c.is_whitespace() && !in_string => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }

            // =====================
            // DEFAULT
            // =====================
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

fn help() {
    println!(
        r#"    
MiniDB SQL Syntax Specification (Supported Features)

    1. REPL / SYSTEM
    exit
    quit
    :q

    2. DATABASE
    create database <name>
    drop database <name>
    Use Database
    use <name>
    use database <name>
    show databases
    show current database

    3. TABLE
    create table <table>
    drop table <table>
    show tables
    Describe
    describe <table>

    4. ALTER TABLE
    * Add Column (Single / Multiple)
    alter table <table> add column <col> <type>
    alter table <table> add column <col1> <type1>, <col2> <type2>

    * Enum Type
    alter table tes add column status enum(lulus, gagal)

    * With Constraint
    alter table tes add column id int primarykey
    alter table tes add column name str notnull
    alter table tes add column age int default 10
    alter table tes add column code int unique
    alter table tes add column id int increment

    * Drop Column
    alter table <table> drop column <col>
    alter table <table> drop column <col1>, <col2>

    5. INSERT
    * With Column List
    insert into <table> (col1, col2) values (v1, v2)

    * Without Column List (Full Row)
    insert into <table> values (v1, v2, v3)

    * Multi Row Insert
    insert into <table> (col1, col2) values (v1, v2), (v3, v4)

    6. SELECT
    * Select All
    select * from <table>

    * Select Columns
    select col1, col2 from <table>

    * With WHERE
    select * from <table> where col = value
    select col1 from <table> where col = value

    7. UPDATE
    * Update All Rows
    update <table> set col = value

    * Update With WHERE
    update <table> set col = value where id = 1

    * Multiple Assignment
    update <table> set col1 = v1, col2 = v2 where id = 1

    8. DELETE
    * Delete With WHERE
    delete from <table> where col = value
    delete from <table> where col1 = v1 and col2 = v2

    * Delete All Rows
    delete from <table>

    9. WHERE CLAUSE

    Comparison Operators

    =
    !=
    <
    >
    <=
    >=

    Logical Operator

    and

    NULL Handling

    col IS NULL
    col IS NOT NULL

    ❌ Invalid (Ditolak)

    col = null
    col != null

    ---

    📊 10. VALUE TYPES

    Integer
    1
    100

    Float
    3.14

    String
    "hello"
    Null
    null

    Enum Value
    gagal
    "gagal"

    ---

    ⚙️ 11. DATA TYPES

    int
    float
    str
    string
    enum(a, b, c)

    ---

    📌 12. CONSTRAINTS

    primarykey
    unique
    notnull
    default <value>
    increment

    ---

    🚫 13. NOT SUPPORTED (CURRENT LIMITATION)

    WHERE

    or
    ( grouping )
    like
    in (...)
    between

    SELECT

    order by
    limit
    alias (as)
    aggregate (count, sum, dll)

    INSERT

    default values

    ALTER

    rename column
    modify column

    ADVANCED

    join
    subquery
    group by
    having

    ---

    🧾 SUMMARY
    Engine saat ini mendukung:

    - ✅ CRUD lengkap (INSERT, SELECT, UPDATE, DELETE)
    - ✅ WHERE dengan AND
    - ✅ ENUM type
    - ✅ DEFAULT value
    - ✅ NULL handling (IS NULL)
    - ✅ ALTER TABLE (ADD / DROP column)

    "#
    )
}
