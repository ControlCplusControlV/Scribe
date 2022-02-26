use colored::*;
use std::path::Path;

use scribe::executor;
use scribe::miden_generator;
use scribe::parser;
use scribe::repl::start_repl;

use scribe::types::expressions_to_tree;
use std::fs;
use std::io::{stdin, stdout, Read, Write};

#[macro_use(quickcheck)]
extern crate quickcheck_macros;

#[macro_use]
extern crate insta;
use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Me")]
struct Opts {
    #[clap(subcommand)]
    subcmd: Option<SubCommand>,
}

#[derive(Parser)]
enum SubCommand {
    Repl {
        #[clap(short, long)]
        functions_file: Option<String>,
        #[clap(short, long)]
        stack: Option<String>,
    },
}

fn print_title(s: &str) {
    let s1 = format!("=== {} ===", s).blue().bold();
    println!("{}", s1);
    println!(" ");
}
fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

fn clear_screen() {
    print!("{esc}c", esc = 27 as char);
}

fn main() {
    let opts: Opts = Opts::parse();
    // opts.skip_setup = true;

    //Match any command line arguments
    match opts.subcmd {
        //If the program is run with "repl" as an argument, start the Miden repl in the terminal
        Some(SubCommand::Repl {
            functions_file,
            stack,
        }) => {
            start_repl(functions_file, stack);
        }
        //If there are no command line arguments, run scribe on the yul contracts in the contracts directory and print the ouput in the terminal
        None => {
            let yul_contracts = read_yul_contracts();

            //For each contract in Vec of YulFile
            for yul_code in yul_contracts {
                //Print the YulFile contents
                clear_screen();
                print_title("Input File");
                println!("{}", yul_code.file_contents);
                pause();
                //Parse the Yul code into a vec of Scribe expressions (Expr)
                let inputs = vec![];
                let parsed = parser::parse_yul_syntax(&yul_code.file_contents);

                //Convert the vec of Expr into an abstract syntax tree
                clear_screen();
                print_title("Parsed Expressions");
                println!("{}", expressions_to_tree(&parsed));
                println!();
                pause();

                //Transpile the AST into Miden assembly
                clear_screen();
                let miden_code = miden_generator::transpile_program(parsed);
                print_title("Generated Miden Assembly");
                println!("{}", miden_code);
                println!();
                pause();

                //Display the Miden output
                clear_screen();
                print_title("Miden Output");
                let execution_value = executor::execute(miden_code, inputs).unwrap();
                let stack = execution_value.last_stack_state();
                let last_stack_value = stack.first().unwrap();
                println!("{}", last_stack_value);
                pause();
            }
        }
    }
}

//Struct to represent a YulFile
pub struct YulFile {
    pub file_name: String,
    pub file_contents: String,
}

//Read in all of the Yul contracts from the contracts directory and return a Vec of Yul Files
fn read_yul_contracts() -> Vec<YulFile> {
    let mut yul_files: Vec<YulFile> = Vec::new();
    let mut paths: Vec<_> = fs::read_dir("../contracts/")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    paths.sort_by_key(|dir| dir.path());
    for path in paths {
        let contents = fs::read_to_string(path.path())
            .expect("Something went wrong readingfrom the contracts directory");

        yul_files.push(YulFile {
            file_name: path.path().display().to_string(),
            file_contents: contents,
        });
    }
    yul_files
}
