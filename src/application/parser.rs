use crate::{
    application::{AppError, Command},
    domain::{Condition, Constraint, DataType, Value},
};

fn tokenize(input: &str) -> Result<Vec<String>, AppError> {
    let mut tokens = Vec::new();
    let mut buf = String::new();
    let mut paren_depth: i32 = 0;
    let mut in_string = false;

    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                buf.push(ch);
                in_string = !in_string;
            }

            '(' if !in_string => {
                paren_depth += 1;
                buf.push(ch);
            }

            ')' if !in_string => {
                paren_depth -= 1;
                if paren_depth < 0 {
                    return Err(AppError::InvalidCommand("unmatched ')'".into()));
                }
                buf.push(ch);
            }

            ' ' if !in_string && paren_depth == 0 => {
                if !buf.is_empty() {
                    tokens.push(buf.clone());
                    buf.clear();
                }
            }

            ',' if !in_string && paren_depth == 0 => {
                if !buf.is_empty() {
                    tokens.push(buf.clone());
                    buf.clear();
                }
            }

            _ => buf.push(ch),
        }
    }

    if in_string {
        return Err(AppError::InvalidCommand(
            "unterminated string literal".into(),
        ));
    }

    if !buf.is_empty() {
        tokens.push(buf);
    }

    Ok(tokens)
}

pub fn parse_command(input: &str) -> Result<Command, AppError> {
    let tokens = tokenize(input)?;
    let refs: Vec<&str> = tokens.iter().map(|s| s.as_str()).collect();

    match refs.as_slice() {
        // =========================================================
        // EXIT / QUIT
        // =========================================================
        ["exit"] | ["quit"] | ["/q"] | ["EXIT"] | ["QUIT"] => Ok(Command::Exit),

        // =========================================================
        // DATABASE
        //
        // Contoh:
        //   create database mydb
        //   use database mydb
        //   drop database mydb
        //   show databases
        //   show current
        // =========================================================
        ["create", "database", name] | ["CREATE", "DATABASE", name] => {
            Ok(Command::CreateDatabase {
                name: name.to_string(),
            })
        }

        ["use", "database", name] | ["USE", "DATABASE", name] | ["use", name] | ["USE", name] => {
            Ok(Command::UseDatabase {
                name: name.to_string(),
            })
        }

        ["drop", "database", name] | ["DROP", "DATABASE", name] => Ok(Command::DropDatabase {
            name: name.to_string(),
        }),

        ["show", "current"] => Ok(Command::ShowCurrentDatabase),

        ["show", "databases"] => Ok(Command::ShowDatabases),

        // =========================================================
        //  TABLE
        //
        // Contoh:
        //   create table users
        //   drop table users
        //   show tables
        //   describe users
        // =========================================================
        ["create", "table", name] => Ok(Command::CreateTable {
            name: name.to_string(),
        }),

        ["drop", "table", name] => Ok(Command::DropTable {
            name: name.to_string(),
        }),

        ["show", "tables"] => Ok(Command::ShowTables),

        ["describe", table] => Ok(Command::DescribeTable {
            table: table.to_string(),
        }),

        // =========================================================
        // COLUMN
        //
        // Contoh:
        //   alter table users add column name str not null
        //   alter table users drop column name
        // =========================================================
        ["alter", "table", table, "add", "column", rest @ ..] => {
            parse_alter_add_column(table, rest)
        }
        ["alter", "table", table, "drop", "column", rest @ ..] => {
            parse_alter_drop_column(table, rest)
        }

        // =========================================================
        // INSERT ROW
        //
        // Contoh:
        //   insert into users (id,name) values (1,"alice")
        // =========================================================
        ["insert", "into", table, cols, "values", vals] => {
            let columns = parse_list(cols)?;
            let values = parse_list(vals)?;

            if columns.len() != values.len() {
                return Err(AppError::InvalidCommand(
                    "columns and values length mismatch".into(),
                ));
            }

            let mut pairs = Vec::new();
            for (c, v) in columns.into_iter().zip(values.into_iter()) {
                pairs.push((c, parse_value(&v)?));
            }

            Ok(Command::InsertRow {
                table: table.to_string(),
                values: pairs,
            })
        }

        // =========================================================
        // UPDATE ROW
        //
        // Contoh:
        // update users set age = 21 wherw name = "jono"
        // =========================================================
        ["update", table, rest @ ..] => parse_update(table, rest),

        // =========================================================
        // DELETE ROW
        //
        // Contoh:
        // delete from users where name = "jojon"
        // =========================================================
        ["delete", "from", table, "where", ..] => {
            let condition = parse_condition(&tokens[3..])?;
            Ok(Command::DeleteWhere {
                table: table.to_string(),
                conditions: vec![condition],
            })
        }

        // =========================================================
        // SELECT ALL
        //
        // Contoh:
        //   select * from users
        // =========================================================
        ["select", "*", "from", table] => Ok(Command::SelectAll {
            table: table.to_string(),
        }),

        // =========================================================
        // SELECT WITH WHERE
        //
        // Contoh:
        //   select * from users where id = 1
        //   select * from users where age > 18
        // =========================================================
        ["select", "*", "from", table, "where", ..] => {
            let condition = parse_condition(&tokens[4..])?;
            Ok(Command::SelectWhere {
                table: table.to_string(),
                condition,
            })
        }

        // =========================================================
        // FALLBACK
        // =========================================================
        _ => Err(AppError::InvalidCommand(input.to_string())),
    }
}

fn parse_update(table: &str, tokens: &[&str]) -> Result<Command, AppError> {
    // cari posisi "set" dan "where"
    let set_pos = tokens
        .iter()
        .position(|t| *t == "set")
        .ok_or_else(|| AppError::InvalidCommand("expected SET".into()))?;

    let where_pos = tokens.iter().position(|t| *t == "where");

    let assign_tokens = match where_pos {
        Some(w) => &tokens[set_pos + 1..w],
        None => &tokens[set_pos + 1..],
    };

    let cond_tokens = match where_pos {
        Some(w) => &tokens[w + 1..],
        None => &[],
    };

    let assignments = parse_assignments(assign_tokens)?;
    let conditions = if cond_tokens.is_empty() {
        Vec::new()
    } else {
        parse_conditions(cond_tokens)?
    };

    Ok(Command::UpdateWhere {
        table: table.to_string(),
        assignments,
        conditions,
    })
}

fn parse_assignments(tokens: &[&str]) -> Result<Vec<(String, Value)>, AppError> {
    let mut result = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        if i + 2 >= tokens.len() || tokens[i + 1] != "=" {
            return Err(AppError::InvalidCommand("invalid assignment".into()));
        }

        let col = tokens[i].to_string();
        let val = parse_value(tokens[i + 2])?;

        result.push((col, val));
        i += 3;
    }

    if result.is_empty() {
        return Err(AppError::InvalidCommand("no assignments".into()));
    }

    Ok(result)
}
fn parse_conditions(tokens: &[&str]) -> Result<Vec<Condition>, AppError> {
    // sementara: satu kondisi
    if tokens.len() != 3 {
        return Err(AppError::InvalidCommand("invalid where clause".into()));
    }

    let col = tokens[0];
    let op = tokens[1];
    let val = parse_value(tokens[2])?;

    let cond = match op {
        "=" => Condition::eq(col, val),
        ">" => Condition::gt(col, val),
        "<" => Condition::lt(col, val),
        _ => return Err(AppError::InvalidCommand(format!("invalid operator {}", op))),
    };

    Ok(vec![cond])
}

fn parse_alter_drop_column(table: &str, rest: &[&str]) -> Result<Command, AppError> {
    if rest.is_empty() {
        return Err(AppError::InvalidCommand("expected column name".into()));
    }

    let columns = rest
        .iter()
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect::<Vec<_>>();

    Ok(Command::AlterTableDropColumn {
        table: table.to_string(),
        columns,
    })
}

fn parse_alter_add_column(table: &str, tokens: &[&str]) -> Result<Command, AppError> {
    let mut columns = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        // minimal: name + type
        if i + 1 >= tokens.len() {
            return Err(AppError::InvalidCommand(
                "column definition incomplete".into(),
            ));
        }

        let name = tokens[i];
        let ty = parse_data_type(tokens[i + 1])?;

        i += 2;

        let mut cons = Vec::new();
        while i < tokens.len() {
            // stop jika token terlihat seperti awal kolom baru
            if looks_like_column_start(tokens, i) {
                break;
            }
            cons.push(tokens[i]);
            i += 1;
        }

        let constraints = parse_constraints(&cons)?;
        columns.push((name.to_string(), ty, constraints));
    }

    Ok(Command::AlterTableAddColumn {
        table: table.to_string(),
        columns,
    })
}
fn looks_like_column_start(tokens: &[&str], i: usize) -> bool {
    i + 1 < tokens.len() && matches!(tokens[i + 1], "int" | "str" | "float")
}

// ===========

fn parse_data_type(token: &str) -> Result<DataType, AppError> {
    match token {
        "int" => Ok(DataType::Int),
        "str" => Ok(DataType::Str),
        "float" => Ok(DataType::Float),
        _ if token.starts_with("enum") => parse_enum_type(token),
        _ => Err(AppError::InvalidCommand(format!(
            "unknown data type: {}",
            token
        ))),
    }
}
fn parse_enum_type(token: &str) -> Result<DataType, AppError> {
    let raw = token
        .strip_prefix("enum")
        .ok_or_else(|| AppError::InvalidCommand("invalid enum type".into()))?;

    let variants = parse_list(raw)?;

    if variants.is_empty() {
        return Err(AppError::InvalidCommand(
            "enum must have at least one variant".into(),
        ));
    }

    Ok(DataType::Enum { variants })
}

fn parse_condition(tokens: &[String]) -> Result<Condition, AppError> {
    // WHERE col op value
    if tokens.len() != 4 || tokens[0] != "where" {
        return Err(AppError::InvalidCommand("invalid where clause".into()));
    }

    let column = &tokens[1];
    let op = &tokens[2];
    let raw_value = &tokens[3];

    let value = parse_value(raw_value)?;

    let condition = match op.as_str() {
        "=" => Condition::eq(column, value),
        "<" => Condition::lt(column, value),
        ">" => Condition::gt(column, value),
        _ => {
            return Err(AppError::InvalidCommand(format!(
                "invalid operator: {}",
                op
            )));
        }
    };

    Ok(condition)
}

fn parse_list(raw: &str) -> Result<Vec<String>, AppError> {
    if !raw.starts_with('(') || !raw.ends_with(')') {
        return Err(AppError::InvalidCommand(
            "expected parenthesized list".into(),
        ));
    }

    let inner = &raw[1..raw.len() - 1];

    Ok(inner
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect())
}

fn parse_value(token: &str) -> Result<Value, AppError> {
    // string literal
    if token.starts_with('"') && token.ends_with('"') {
        let inner = &token[1..token.len() - 1];
        return Ok(Value::Str(inner.to_string()));
    }

    // int
    if let Ok(n) = token.parse::<i64>() {
        return Ok(Value::Int(n));
    }

    // float
    if let Ok(f) = token.parse::<f64>() {
        return Ok(Value::Float(f));
    }

    // null bukan tipe untuk input
    // // null / absen
    // if token == "null" {
    //     return Ok(Value::Absen(true));
    // }

    // Handle enum("value") atau enum(value)
    if token.starts_with("enum(") && token.ends_with(')') {
        let inner = &token[5..token.len() - 1];
        // Opsional: bersihkan tanda kutip jika di dalam enum ada kutip, misal enum("lulus")
        let clean_inner = inner.trim_matches('"');

        if is_valid_identifier(clean_inner) {
            return Ok(Value::Enum {
                value: clean_inner.to_string(),
            });
        }
    }

    // enum literal (identifier)
    if is_valid_identifier(token) {
        return Ok(Value::Enum {
            value: token.to_string(),
        });
    }

    Err(AppError::InvalidCommand(format!(
        "invalid value: {}",
        token
    )))
}

fn is_valid_identifier(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }

    chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn parse_constraints(tokens: &[&str]) -> Result<Vec<Constraint>, AppError> {
    let mut constraints = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        match tokens[i] {
            "not" => {
                let next = tokens.get(i + 1);
                if next != Some(&"null") {
                    return Err(AppError::InvalidCommand("expected `not null`".into()));
                }

                constraints.push(Constraint::NotNull);
                i += 2;
            }

            "null" => {
                constraints.push(Constraint::Nullable);
                i += 1;
            }

            // "unique" | "uniq" => {
            //     constraints.push(Constraint::Unique);
            //     i += 1;
            // }
            "primary" => {
                let next = tokens.get(i + 1);
                if next != Some(&"key") {
                    return Err(AppError::InvalidCommand("expected `primary key`".into()));
                }

                constraints.push(Constraint::Unique);
                i += 2;
            }

            "default" => {
                let next = tokens.get(i + 1).ok_or_else(|| {
                    AppError::InvalidCommand("expected value after `default`".into())
                })?;

                let value = parse_value(next)?;
                constraints.push(Constraint::Default(value));
                i += 2;
            }

            other => {
                return Err(AppError::InvalidCommand(format!(
                    "unknown constraint: {}",
                    other
                )));
            }
        }
    }

    Ok(constraints)
}
