use rustyline::DefaultEditor;

use crate::{
    application::{AppManager, QueryInfo, executor, parser, print_query_error, print_query_result},
    storage::DatabaseStorage,
};

pub fn start<S>(app: &mut AppManager<S>)
where
    S: DatabaseStorage,
{
    print_welcome();
    let mut rl = DefaultEditor::new().unwrap();

    loop {
        // PROMPT
        let prompt = if let Ok(db_name) = app.show_current_database() {
            format!("[minisql:{}] > ", db_name)
        } else {
            "minisql > ".to_string()
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

                let cmd = match parser::parse(tokens) {
                    Ok(c) => c,
                    Err(e) => {
                        print_query_error(e);
                        continue;
                    }
                };

                if cmd.is_exit() {
                    print_query_result(QueryInfo::Exit);
                    break;
                }

                match executor::execute(cmd, app) {
                    Ok(info) => print_query_result(info),
                    Err(err) => print_query_error(err),
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

pub fn print_welcome() {
    println!("╔══════════════════════════════════╗");
    println!("║      MINI DB TOY SQL ENGINE      ║");
    println!("║              v0.0.1              ║");
    println!("╚══════════════════════════════════╝");
    println!("Type 'exit' to quit");
    println!();
}
