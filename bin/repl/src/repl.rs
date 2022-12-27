use papyrus::executor::execute;
use papyrus::utils::load_all_procs;
use colored::*;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
use std::path::Path;

//The Scribe Read–eval–print loop or repl for short is a Miden shell that allows for quick and easy debugging with Miden assembly!
//To use the repl, simply type "scribe repl" when in the transpiler crate and the repl will launch.
//Now that you have the repl launched, there are a bunch of awesome things you can do like execute any Miden instruction, use procedures,
//undo executed instructions, check the stack at anytime and more! Check out the list of commands that you can use below. After exiting the
// repl, a history.txt file will be saved

//Miden Instructions
//  Any Miden instruction included in the Miden Assembly HackMD is valid. (Ex. push.0, drop, dropw, swap, cswap, shr, mem.load.n, ect.)
//  You can input instructions one by one or multiple instructions in one input.
//  Ex.
//  push.1
//  push.2
//  push.3
//  Is the same as
//  push.1 push.2 push.3

//`stack`
//  Use the `stack` command to check the state of the stack at anytime. When you start the repl, the stack will be empty.
//  Try pushing some values and checking the stack!
//  Ex.
//  push.1 push.2 push.3
//  stack
//  >> 3 2 1 0 0 0 0 0 0 0 0 0 0 0 0 0

//`undo`
//  Use the `undo` command at anytime to revert to the last state of the stack before a command or Miden instruction. You can use `undo`
//  as many times as you want to restore the state of a stack n instructions ago.
//  Ex. push.1 push.2 push.3
//  stack
//  >> 3 2 1 0 0 0 0 0 0 0 0 0 0 0 0 0
//  push.4
//  stack
//  >> 4 3 2 1 0 0 0 0 0 0 0 0 0 0 0 0
//  push.5
//  stack
//  >> 5 4 3 2 1 0 0 0 0 0 0 0 0 0 0 0
//  undo
//  stack
//  >> 4 3 2 1 0 0 0 0 0 0 0 0 0 0 0 0
//  undo
//  stack
//  >> 3 2 1 0 0 0 0 0 0 0 0 0 0 0 0 0

//`program`
//  Use the `program` command at anytime to see the full Miden assembly that you have input to that point as a Miden program
//  Ex.
//  push.1
//  push.2
//  push.3
//  add
//  add
//  program
// >>
//  begin
//      push.1
//      push.2
//      push.3
//      add
//      add
//  end

//`help`
// Use the `help` command at any time to see a list of available commands.

pub fn start_repl(functions_file: Option<String>, stack_string: Option<String>) {
    let mut program_lines: Vec<String> = Vec::new();
    let mut functions_miden = "".to_string();
    if let Some(functions_file) = functions_file {
        let path = Path::new(&functions_file);
        functions_miden = fs::read_to_string(path).expect("Something went wrong reading the file");
    }
    if let Some(stack_string) = stack_string {
        program_lines.push(
            stack_string
                .split(',')
                .map(|s| format!("push.{}", s))
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join(" "),
        );
    }
    let mut rl = Editor::<()>::new();
    loop {
        let program = format!(
            "begin\n{}\nend",
            program_lines
                .iter()
                .map(|l| format!("    {}", l))
                .collect::<Vec<_>>()
                .join("\n")
        );
        let program_with_procs = format!(
            "{}\n{}\n{}",
            functions_miden,
            load_all_procs(),
            program.clone()
        );
        let result = execute(program_with_procs, vec![]);

        let mut result_string = "".to_string();
        if !program_lines.is_empty() {
            match result {
                Ok(execution_value) => {
                    let stack = execution_value.last_stack_state();
                    result_string = format!(
                        "\n{}",
                        stack
                            .iter()
                            .map(|f| format!("{}", f))
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
        println!();
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