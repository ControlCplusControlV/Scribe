use colored::*;


use scribe::executor;
use scribe::miden_generator;
use scribe::parser;

use scribe::types::expressions_to_tree;
use std::fs;
use std::io::{stdin, stdout, Read, Write};

#[macro_use]
extern crate pest_derive;

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
    let yul_contracts = read_yul_contracts();

    for yul_code in yul_contracts {
        clear_screen();
        print_title("Input File");
        println!("{}", yul_code.file_contents);
        pause();
        let inputs = vec![];
        let parsed = parser::parse_yul_syntax(&yul_code.file_contents);

        clear_screen();
        print_title("Parsed Expressions");
        println!("{}", expressions_to_tree(&parsed));
        println!();
        pause();

        clear_screen();
        let miden_code = miden_generator::transpile_program(parsed);
        print_title("Generated Miden Assembly");
        println!("{}", miden_code);
        println!();
        pause();

        clear_screen();
        print_title("Miden Output");
        let execution_value = executor::execute(miden_code, inputs).unwrap();
        let stack = execution_value.last_stack_state();
        let last_stack_value = stack.first().unwrap();
        println!("{}", last_stack_value);
        pause();
    }
}

pub struct YulFile {
    pub file_name: String,
    pub file_contents: String,
}

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
