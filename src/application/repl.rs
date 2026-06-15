use rustyline::DefaultEditor;

use crate::{
    application::{AppManager, QueryInfo, executor, parser, print_query_error, print_query_result},
    storage::DatabaseStorage,
};

use std::path::PathBuf;

/// Get history file path (~/.minidb/history.txt)
fn get_history_path() -> Result<PathBuf, std::io::Error> {
    // Allow override via environment variable
    if let Ok(custom_path) = std::env::var("MINIDB_HOME") {
        let minidb_dir = PathBuf::from(custom_path);
        std::fs::create_dir_all(&minidb_dir)?;
        return Ok(minidb_dir.join("history.txt"));
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let minidb_dir = PathBuf::from(home).join(".minidb");
    std::fs::create_dir_all(&minidb_dir)?;
    Ok(minidb_dir.join("history.txt"))
}

pub fn start<S: DatabaseStorage>(app: &mut AppManager<S>) {
    print_welcome();
    let mut rl = DefaultEditor::new().unwrap();

    // Load history from ~/.minidb/history.txt
    let history_path = get_history_path();
    if let Ok(history_path) = &history_path {
        let _ = rl.load_history(history_path);
    }

    loop {
        // PROMPT
        let prompt = if let Ok(db_name) = app.show_current_database() {
            format!("[minisql:{}] > ", db_name)
        } else {
            "minisql > ".to_string()
        };

        // Read multi-line input
        let input = match read_multi_line(&mut rl, &prompt) {
            Ok(input) => input,
            Err(err) => match err {
                rustyline::error::ReadlineError::Interrupted => {
                    println!("^C (Type 'exit' to quit)");
                    continue;
                }
                rustyline::error::ReadlineError::Eof => {
                    println!("\nbye");
                    break;
                }
                _ => {
                    println!("error: {:?}", err);
                    break;
                }
            },
        };

        if input.is_empty() {
            continue;
        }

        // Add to history (only first line of multi-line query)
        let first_line = input.lines().next().unwrap_or(&input);
        let _ = rl.add_history_entry(first_line);

        // Remove trailing semicolon
        let input = trim_semicolon(&input);

        // TOKENIZE AND PARSE
        let tokens = tokenize(&input);

        let cmd = match parser::parse(tokens) {
            Ok(c) => c,
            Err(e) => {
                print_query_error(e);
                continue;
            }
        };

        if cmd.is_exit() {
            // Save history to ~/.minidb/history.txt before exit
            if let Ok(history_path) = history_path {
                let _ = rl.save_history(&history_path);
            }
            print_query_result(QueryInfo::Exit);
            break;
        }

        match executor::execute(cmd, app) {
            Ok(info) => print_query_result(info),
            Err(err) => print_query_error(err),
        }
    }
}

/// Read multi-line input until query is complete
fn read_multi_line(
    rl: &mut DefaultEditor,
    prompt: &str,
) -> Result<String, rustyline::error::ReadlineError> {
    let mut lines = Vec::new();
    let mut line_num = 0;

    loop {
        let current_prompt = if line_num == 0 {
            prompt.to_string()
        } else {
            // Continuation prompt
            let indent = " ".repeat(prompt.len() - 3);
            format!("{} -> ", indent)
        };

        let line = rl.readline(&current_prompt)?;
        let line = line.trim_end();

        lines.push(line.to_string());

        // Check if query is complete
        let combined = lines.join(" ");
        if is_query_complete(&combined) {
            return Ok(combined);
        }

        line_num += 1;
    }
}

/// Check if SQL query is complete (not expecting more input)
fn is_query_complete(input: &str) -> bool {
    let input = input.trim();

    if input.is_empty() {
        return false;
    }

    // Count unclosed parentheses
    let mut paren_depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for c in input.chars() {
        match c {
            '"' if !escaped => in_string = !in_string,
            '(' if !in_string => paren_depth += 1,
            ')' if !in_string => {
                if paren_depth > 0 {
                    paren_depth -= 1;
                }
            }
            '\\' => escaped = true,
            _ => escaped = false,
        }
    }

    // Query is complete if:
    // - No unclosed parentheses
    // - Not in string
    // - Ends with semicolon or is a complete command
    let has_semicolon = input.ends_with(';');
    let is_complete_command = matches!(
        input.to_lowercase().as_str(),
        "exit" | "quit" | "help" | "/q" | ":q"
    );

    (paren_depth == 0 && !in_string) && (has_semicolon || is_complete_command)
}

/// Remove trailing semicolon
fn trim_semicolon(input: &str) -> String {
    let input = input.trim();
    if input.ends_with(';') {
        input[..input.len() - 1].to_string()
    } else {
        input.to_string()
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
            // MULTI-CHAR OPERATORS
            // =====================
            '>' | '<' | '!' if !in_string => {
                if !current.trim().is_empty() {
                    tokens.push(current.trim().to_string());
                    current.clear();
                }

                if let Some('=') = chars.peek() {
                    tokens.push(format!("{}=", c));
                    chars.next(); // consume '='
                } else {
                    tokens.push(c.to_string());
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
