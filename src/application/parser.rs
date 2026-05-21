use crate::{
    application::{AppError, Command, map_domain_error},
    domain::{CompareExpr, CompareOp, Constraint, DataType, Expr, Value},
};

// Case-insensitive keywords, case-sensitive identifiers
const KEYWORDS: &[&str] = &[
    "select",
    "from",
    "where",
    "insert",
    "into",
    "values",
    "update",
    "set",
    "delete",
    "create",
    "drop",
    "table",
    "tables",
    "database",
    "databases",
    "show",
    "use",
    "alter",
    "add",
    "column",
    "describe",
    "and",
    "or",
    "not",
    "is",
    "null",
    //constraint
    "notnull",
    "primarykey",
    "increment",
    "default",
];

fn normalize_keywords(tokens: Vec<String>) -> Vec<String> {
    tokens
        .into_iter()
        .map(|t| {
            if KEYWORDS.iter().any(|kw| t.eq_ignore_ascii_case(kw)) {
                t.to_ascii_lowercase()
            } else {
                t
            }
        })
        .collect()
}

fn normalize_command(cmd: &str) -> String {
    match cmd.to_lowercase().as_str() {
        "exit" | "quit" | "/q" | ":q" => "exit".into(),
        "help" | "/h" | ":?" => "help".into(),
        _ => cmd.into(),
    }
}

pub fn parse(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.is_empty() {
        return Err(AppError::InvalidCommand("empty input".into()));
    }

    let tokens = normalize_keywords(tokens);

    let first = normalize_command(&tokens[0]);

    match first.as_str() {
        "exit" => Ok(Command::Exit),
        "help" => Ok(Command::Help),
        "debug" => parse_debug(tokens),
        "select" => parse_select(tokens),
        "use" => parse_use(tokens),
        "show" => parse_show(tokens),
        "create" => parse_create(tokens),
        "drop" => parse_drop(tokens),
        "insert" => parse_insert(tokens),
        "update" => parse_update(tokens),
        "delete" => parse_delete(tokens),
        "alter" => parse_alter(tokens),
        "describe" => parse_describe(tokens),
        _ => Err(AppError::InvalidCommand("unknown command".into())),
    }
}

fn parse_debug(tokens: Vec<String>) -> Result<Command, AppError> {
    let t: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
    match t.as_slice() {
        ["debug", "database"] => Ok(Command::DebugDatabase),
        ["debug", "table", name] => Ok(Command::DebugTable {
            name: name.to_string(),
        }),
        _ => Err(AppError::InvalidCommand("Not database selected".into())),
    }
}

fn parse_insert(tokens: Vec<String>) -> Result<Command, AppError> {
    let mut i = 0;

    // =====================
    // INSERT INTO <table>
    // =====================
    expect_token(&tokens, &mut i, "insert")?;
    expect_token(&tokens, &mut i, "into")?;

    let table = tokens
        .get(i)
        .ok_or(AppError::InvalidCommand("missing table".into()))?
        .clone();
    i += 1;

    // =====================
    // OPTIONAL COLUMN LIST
    // =====================
    let columns = if matches!(tokens.get(i).map(|s| s.as_str()), Some("(")) {
        let (cols, next_i) = parse_column_list(&tokens, i)?;
        i = next_i;
        Some(cols)
    } else {
        None
    };

    // =====================
    // VALUES
    // =====================
    expect_token(&tokens, &mut i, "values")?;

    // =====================
    // MULTI ROW
    // =====================
    let rows = parse_multi_values(&tokens[i..])?;

    Ok(Command::InsertRow {
        table,
        columns,
        rows,
    })
}

fn parse_column_list(tokens: &[String], start: usize) -> Result<(Vec<String>, usize), AppError> {
    let mut cols = Vec::new();
    let mut i = start + 1;

    while let Some(t) = tokens.get(i) {
        if t == ")" {
            return Ok((cols, i + 1));
        }

        if t != "," {
            cols.push(t.clone());
        }

        i += 1;
    }

    Err(AppError::InvalidCommand("unclosed column list".into()))
}

fn expect_token(tokens: &[String], i: &mut usize, expected: &str) -> Result<(), AppError> {
    match tokens.get(*i).map(|s| s.as_str()) {
        Some(t) if t == expected => {
            *i += 1;
            Ok(())
        }
        _ => Err(AppError::InvalidCommand(format!("expected {}", expected))),
    }
}

fn parse_multi_values(tokens: &[String]) -> Result<Vec<Vec<Value>>, AppError> {
    let mut rows = Vec::new();
    let mut current = Vec::new();
    let mut depth = 0;

    for t in tokens {
        match t.as_str() {
            "(" => {
                depth += 1;

                if depth == 1 {
                    current = Vec::new();
                }
            }

            ")" => {
                if depth == 0 {
                    return Err(AppError::InvalidCommand("unexpected ')'".into()));
                }

                depth -= 1;

                if depth == 0 {
                    if current.is_empty() {
                        return Err(AppError::InvalidCommand("empty row".into()));
                    }

                    rows.push(std::mem::take(&mut current));
                }
            }

            "," => {
                if depth == 1 {
                    continue; // separator antar value
                }
            }

            _ => {
                if depth >= 1 {
                    current.push(parse_value(t)?);
                }
            }
        }
    }

    if depth != 0 {
        return Err(AppError::InvalidCommand("unbalanced parentheses".into()));
    }

    if rows.is_empty() {
        return Err(AppError::InvalidCommand("no values provided".into()));
    }

    Ok(rows)
}

fn parse_alter(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() < 6 {
        return Err(AppError::InvalidCommand("invalid alter".into()));
    }

    if tokens[1] != "table" {
        return Err(AppError::InvalidCommand("invalid alter".into()));
    }

    let table = tokens[2].clone();

    match tokens[3].as_str() {
        "add" => parse_alter_add(table, &tokens[4..]),
        "drop" => parse_alter_drop(table, &tokens[4..]),
        _ => Err(AppError::InvalidCommand("invalid alter".into())),
    }
}

fn parse_alter_add(table: String, tokens: &[String]) -> Result<Command, AppError> {
    if tokens.get(0).map(|s| s.as_str()) != Some("column") {
        return Err(AppError::InvalidCommand("expected 'column'".into()));
    }

    let raw_columns = &tokens[1..];

    let groups = split_columns(raw_columns);

    let mut columns = Vec::new();

    for g in groups {
        columns.push(parse_single_column(&g)?);
    }

    Ok(Command::AlterTableAddColumn { table, columns })
}

fn parse_alter_drop(table: String, tokens: &[String]) -> Result<Command, AppError> {
    if tokens.get(0).map(|s| s.as_str()) != Some("column") {
        return Err(AppError::InvalidCommand("expected 'column'".into()));
    }

    let columns = parse_columns(&tokens[1..])?;

    Ok(Command::AlterTableDropColumn { table, columns })
}

fn split_columns(tokens: &[String]) -> Vec<Vec<String>> {
    let mut result = Vec::new();
    let mut current = Vec::new();
    let mut paren_level = 0;

    for t in tokens {
        match t.as_str() {
            "(" => {
                paren_level += 1;
                current.push(t.clone());
            }
            ")" => {
                paren_level -= 1;
                current.push(t.clone());
            }
            "," if paren_level == 0 => {
                if !current.is_empty() {
                    result.push(current);
                    current = Vec::new();
                }
            }
            _ => current.push(t.clone()),
        }
    }

    if !current.is_empty() {
        result.push(current);
    }

    result
}

fn parse_single_column(tokens: &[String]) -> Result<(String, DataType, Vec<Constraint>), AppError> {
    let mut i = 0;

    let name = tokens[i].clone();
    i += 1;

    // =====================
    // DATATYPE
    // =====================
    let dtype = if tokens[i] == "enum" {
        i += 1;

        if tokens.get(i).map(|s| s.as_str()) != Some("(") {
            return Err(AppError::InvalidCommand("expected '(' after enum".into()));
        }

        i += 1;

        let mut variants = Vec::new();

        while let Some(t) = tokens.get(i) {
            if t == ")" {
                i += 1; // INI WAJIB (skip ')')
                break;
            }

            if t != "," {
                variants.push(t.clone());
            }

            i += 1;
        }

        DataType::enum_of(variants).map_err(map_domain_error)?
    } else {
        let dt = parse_datatype(tokens.get(i))
            .map_err(|_| AppError::InvalidCommand("invalid datatype".into()))?;

        i += 1;

        dt
    };

    // =====================
    // CONSTRAINT
    // =====================
    let mut constraints = Vec::new();

    while let Some(t) = tokens.get(i) {
        match t.as_str() {
            "unique" => {
                constraints.push(Constraint::Unique);
                i += 1;
            }

            "notnull" => {
                constraints.push(Constraint::NotNull);
                i += 1;
            }

            "default" => {
                let val_token = tokens
                    .get(i + 1)
                    .ok_or(AppError::InvalidCommand("missing default value".into()))?;

                let raw = parse_value(val_token)?;
                let coerced = dtype
                    .coerce_value(raw)
                    .map_err(|_| AppError::InvalidCommand("invalid default value".into()))?;

                constraints.push(Constraint::Default(coerced));
                i += 2;
            }

            "primarykey" => {
                constraints.push(Constraint::PrimaryKey);
                i += 1;
            }

            "increment" => {
                constraints.push(Constraint::Increment);
                i += 1;
            }

            _ => {
                return Err(AppError::InvalidCommand(format!(
                    "unknown constraint: {}",
                    t
                )));
            }
        }
    }

    Ok((name, dtype, constraints))
}

fn parse_datatype(token: Option<&String>) -> Result<DataType, AppError> {
    let t = token.ok_or(AppError::InvalidCommand("missing datatype".into()))?;

    match t.to_lowercase().as_str() {
        "int" => Ok(DataType::Int),
        "float" => Ok(DataType::Float),
        "string" | "str" => Ok(DataType::Str),
        _ => Err(AppError::InvalidCommand(format!("unknown datatype {}", t))),
    }
}

fn parse_select(tokens: Vec<String>) -> Result<Command, AppError> {
    let from_pos = tokens
        .iter()
        .position(|t| t == "from")
        .ok_or(AppError::InvalidCommand("missing from".into()))?;

    if from_pos < 2 {
        return Err(AppError::InvalidCommand("invalid select".into()));
    }

    let table = tokens
        .get(from_pos + 1)
        .ok_or(AppError::InvalidCommand("missing table".into()))?
        .clone();

    let tail = &tokens[from_pos + 2..];

    // =====================
    // HANDLE "*"
    // =====================
    if tokens[1] == "*" {
        match tail {
            [] => {
                return Ok(Command::SelectAll { table });
            }

            [kw, rest @ ..] if kw == "where" => {
                let conditions = parse_expr(rest)?;

                return Ok(Command::SelectWhere { table, conditions });
            }

            _ => {
                return Err(AppError::InvalidCommand(
                    "unexpected token after table".into(),
                ));
            }
        }
    }

    // =====================
    // COLUMNS
    // =====================
    let columns = parse_columns(&tokens[1..from_pos])?;

    // =====================
    // HANDLE TAIL
    // =====================
    match tail {
        [] => Ok(Command::SelectColumns { table, columns }),

        [kw, rest @ ..] if kw == "where" => {
            let conditions = parse_expr(rest)?;

            Ok(Command::SelectColumnsWhere {
                table,
                columns,
                conditions,
            })
        }

        _ => Err(AppError::InvalidCommand(
            "unexpected token after table".into(),
        )),
    }
}

fn parse_columns(tokens: &[String]) -> Result<Vec<String>, AppError> {
    let mut cols = Vec::new();
    let mut expect_col = true;

    for t in tokens {
        if t == "," {
            if expect_col {
                return Err(AppError::InvalidCommand("invalid column list".into()));
            }
            expect_col = true;
            continue;
        }

        if !expect_col {
            return Err(AppError::InvalidCommand("invalid column list".into()));
        }

        cols.push(t.clone());
        expect_col = false;
    }

    // kalau terakhir koma → error
    if expect_col {
        return Err(AppError::InvalidCommand("invalid column list".into()));
    }

    Ok(cols)
}

fn parse_expr(tokens: &[String]) -> Result<Expr, AppError> {
    let mut exprs = Vec::new();
    let mut ops = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let column = tokens[i].clone();

        // =====================
        // IS NULL / IS NOT NULL
        // =====================
        let expr = if tokens.get(i + 1).map(|s| s.as_str()) == Some("is") {
            match tokens.get(i + 2).map(|s| s.as_str()) {
                Some("null") => {
                    i += 3;

                    Expr::Compare(CompareExpr {
                        column,
                        op: CompareOp::IsNull,
                        value: None,
                    })
                }

                Some("not") => {
                    if tokens.get(i + 3).map(|s| s.as_str()) != Some("null") {
                        return Err(AppError::InvalidCommand("expected NULL".into()));
                    }

                    i += 4;

                    Expr::Compare(CompareExpr {
                        column,
                        op: CompareOp::IsNotNull,
                        value: None,
                    })
                }

                _ => {
                    return Err(AppError::InvalidCommand("expected NULL".into()));
                }
            }
        }
        // =====================
        // NORMAL OPERATOR
        // =====================
        else {
            if i + 2 >= tokens.len() {
                return Err(AppError::InvalidCommand("invalid where clause".into()));
            }

            let op = match tokens[i + 1].as_str() {
                "=" => CompareOp::Eq,
                "!=" => CompareOp::Ne,
                "<" => CompareOp::Lt,
                ">" => CompareOp::Gt,
                "<=" => CompareOp::Lte,
                ">=" => CompareOp::Gte,

                _ => {
                    return Err(AppError::InvalidCommand("invalid operator".into()));
                }
            };

            let value = parse_value(&tokens[i + 2])?;

            i += 3;

            Expr::Compare(CompareExpr {
                column,
                op,
                value: Some(value),
            })
        };

        exprs.push(expr);

        // =====================
        // HANDLE AND / OR
        // =====================
        if i < tokens.len() {
            match tokens[i].as_str() {
                "and" => ops.push("and"),
                "or" => ops.push("or"),
                _ => {
                    return Err(AppError::InvalidCommand("expected AND or OR".into()));
                }
            }

            i += 1;
        }
    }

    build_expr_tree(exprs, ops)
}
fn build_expr_tree(mut exprs: Vec<Expr>, ops: Vec<&str>) -> Result<Expr, AppError> {
    if exprs.is_empty() {
        return Err(AppError::InvalidCommand("empty expression".into()));
    }

    let mut current = exprs.remove(0);

    for (op, rhs) in ops.into_iter().zip(exprs) {
        current = match op {
            "and" => Expr::And(vec![current, rhs]),
            "or" => Expr::Or(vec![current, rhs]),
            _ => unreachable!(),
        };
    }

    Ok(current)
}

fn parse_value(token: &str) -> Result<Value, AppError> {
    if let Ok(v) = token.parse::<i64>() {
        return Ok(Value::Int(v));
    }

    if let Ok(v) = token.parse::<f64>() {
        return Ok(Value::Float(v));
    }

    if token.eq_ignore_ascii_case("null") {
        return Ok(Value::Null);
    }

    if token.starts_with('"') && token.ends_with('"') {
        return Ok(Value::Str(token.trim_matches('"').to_string()));
    }

    // fallback
    Ok(Value::Str(token.to_string()))
}

// use fill
fn parse_use(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() != 2 {
        return Err(AppError::InvalidCommand("invalid use".into()));
    } else if tokens.len() == 2 {
        let db = tokens[1].clone();
        Ok(Command::UseDatabase { name: db })
    } else {
        let db = tokens[2].clone();
        Ok(Command::UseDatabase { name: db })
    }
}

// show databases
// show tables
// show current database
fn parse_show(tokens: Vec<String>) -> Result<Command, AppError> {
    let t: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();
    match t.as_slice() {
        ["show", "databases"] => Ok(Command::ShowDatabases),
        ["show", "tables"] => Ok(Command::ShowTables),
        ["show", "current", "database"] => Ok(Command::ShowCurrentDatabase),
        _ => Err(AppError::InvalidCommand("invalid show".into())),
    }
}

// create database mydb
fn parse_create(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() < 3 {
        return Err(AppError::InvalidCommand("invalid create".into()));
    }

    match tokens[1].as_str() {
        "table" => Ok(Command::CreateTable {
            name: tokens[2].clone(),
        }),
        "database" => Ok(Command::CreateDatabase {
            name: tokens[2].clone(),
        }),
        _ => Err(AppError::InvalidCommand("invalid create".into())),
    }
}

fn parse_drop(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() < 3 {
        return Err(AppError::InvalidCommand("invalid drop".into()));
    }

    match tokens[1].as_str() {
        "table" => Ok(Command::DropTable {
            name: tokens[2].clone(),
        }),
        "database" => Ok(Command::DropDatabase {
            name: tokens[2].clone(),
        }),
        _ => Err(AppError::InvalidCommand("invalid drop".into())),
    }
}

// describe <table>
fn parse_describe(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() != 2 {
        return Err(AppError::InvalidCommand("invalid describe".into()));
    }

    Ok(Command::DescribeTable {
        table: tokens[1].clone(),
    })
}

// delete from rdb where id = 1
fn parse_delete(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() < 3 || tokens[1] != "from" {
        return Err(AppError::InvalidCommand("invalid delete".into()));
    }

    let table = tokens[2].clone();

    // TANPA WHERE → delete all
    if tokens.len() == 3 {
        return Ok(Command::Delete {
            table,
            conditions: Expr::And(vec![]),
        });
    }

    // ADA TOKEN LANJUTAN → HARUS WHERE
    if tokens[3] != "where" {
        return Err(AppError::InvalidCommand("expected WHERE".into()));
    }

    let conditions = parse_expr(&tokens[4..])?;

    Ok(Command::Delete { table, conditions })
}

// UPDATE users SET name = jono WHERE id = 1
// UPDATE users SET name = jono, status = aktif WHERE id = 1
// UPDATE users SET name = jono
fn parse_update(tokens: Vec<String>) -> Result<Command, AppError> {
    if tokens.len() < 4 {
        return Err(AppError::InvalidCommand("invalid update".into()));
    }

    let table = tokens[1].clone();

    if tokens[2] != "set" {
        return Err(AppError::InvalidCommand("expected SET".into()));
    }

    // =====================
    // CARI WHERE (OPTIONAL)
    // =====================
    let where_pos = tokens.iter().position(|t| t == "where");

    let end_assign = where_pos.unwrap_or(tokens.len());

    // =====================
    // PARSE ASSIGNMENTS
    // =====================
    let mut assignments = Vec::new();
    let mut i = 3;

    while i < end_assign {
        let col = tokens
            .get(i)
            .ok_or(AppError::InvalidCommand("invalid assignment".into()))?;

        let eq = tokens
            .get(i + 1)
            .ok_or(AppError::InvalidCommand("invalid assignment".into()))?;

        let val = tokens
            .get(i + 2)
            .ok_or(AppError::InvalidCommand("invalid assignment".into()))?;

        if eq != "=" {
            return Err(AppError::InvalidCommand("expected '='".into()));
        }

        assignments.push((col.clone(), parse_value(val)?));

        i += 3;

        // skip comma kalau ada
        if let Some(t) = tokens.get(i) {
            if t == "," {
                i += 1;
            }
        }
    }

    // =====================
    // PARSE CONDITIONS (OPTIONAL)
    // =====================
    let conditions = if let Some(pos) = where_pos {
        parse_expr(&tokens[pos + 1..])?
    } else {
        Expr::And(vec![])
    };

    Ok(Command::UpdateWhere {
        table,
        assignments,
        conditions,
    })
}
