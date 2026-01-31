use std::io::{self, Write};

use crate::application::{AppError, AppManager};
use crate::cli::{CommandOutput, execute_command, parse_command};
use crate::storage::DatabaseStorage;

pub fn run_repl<S: DatabaseStorage>(app: &mut AppManager<S>) {
    let stdin = io::stdin();

    loop {
        // prompt
        print!("minidb> ");
        io::stdout().flush().ok();

        // read input
        let mut input = String::new();
        if stdin.read_line(&mut input).is_err() {
            println!("failed to read input");
            continue;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // parse
        let command = match parse_command(input) {
            Ok(cmd) => cmd,
            Err(err) => {
                print_error(err);
                continue;
            }
        };

        // exit
        if command.is_exit() {
            break;
        }

        // execute
        match execute_command(app, command) {
            Ok(output) => print_output(output),
            Err(err) => print_error(err),
        }
    }
}

fn print_output(output: CommandOutput) {
    match output {
        CommandOutput::Ok => println!("ok"),
        CommandOutput::Affected(n) => println!("{n} rows affected"),
        CommandOutput::Rows(rows) => {
            for row in rows {
                println!("{row:?}");
            }
        }
        CommandOutput::Columns(col) => {
            for c in col {
                println!("{c:?}");
            }
        }

        CommandOutput::Message(msg) => println!("{msg}"),
        CommandOutput::Exit => println!("bye"),
    }
}

fn print_error(err: AppError) {
    println!("error: {:#?}", err);
}
