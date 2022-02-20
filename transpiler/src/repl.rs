use crate::executor::execute;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub fn start_repl() {
    let mut program_lines: Vec<String> = Vec::new();
    let mut rl = Editor::<()>::new();
    if rl.load_history("repl_history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let program = format!(
            "begin\n{}\nend",
            program_lines
                .iter()
                .map(|l| format!("    {}", l))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let result = execute(program.clone(), vec![]);
        let mut result_string = "".to_string();
        if program_lines.len() > 0 {
            match result {
                Ok(execution_value) => {
                    let stack = execution_value.last_stack_state();
                    result_string = format!(
                        "\n{}",
                        stack
                            .iter()
                            .map(|f| format!("{}", f.to_string()))
                            .collect::<Vec<_>>()
                            .join(" "),
                    );
                }
                Err(e) => {
                    result_string = format!("Error running program: {:?}", e);
                    println!("{}", result_string);
                    program_lines.pop();
                }
            }
        }
        println!("");
        let readline = rl.readline(&">> ".blue());
        match readline {
            Ok(line) => {
                if line == "program" {
                    println!("\n{}", program);
                } else if line == "help" {
                    println!("Available commands:");
                    println!();
                    println!("stack: display the stack");
                    println!("undo: remove the last instruction");
                    println!("program: display the program");
                } else if line == "undo" {
                    let last_line = program_lines.pop().unwrap();
                    println!("Undoing {}", last_line);
                } else if line == "stack" || line == "res" {
                    println!("{}", result_string);
                } else {
                    rl.add_history_entry(line.clone());
                    program_lines.push(line.clone());
                    // println!("{}", line);
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };
    }
    rl.save_history("history.txt").unwrap();
}
