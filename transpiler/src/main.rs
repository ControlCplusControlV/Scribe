use colored::*;
use scribe::test_utilities::write_yul_to_masm;

use scribe::repl::start_repl;

use scribe::types::YulFile;
use std::fs;
use std::io::{stdin, stdout, Read, Write};
extern crate quickcheck_macros;

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

//Display a "title" in bold when running Scribe examples in the terminal
fn print_title(s: &str) {
    let s1 = format!("=== {} ===", s).blue().bold();
    println!("{}", s1);
    println!(" ");
}

//Wait for the user to press enter to continue when running Scribe examples in the terminal
fn pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

//Clear the terminal when running Scribe examples
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
                write_yul_to_masm(yul_code)
            }
        }
    }
}

//Read in all of the Yul contracts from the contracts directory and return a Vec of Yul Files
fn read_yul_contracts() -> Vec<YulFile> {
    let mut yul_files: Vec<YulFile> = Vec::new();
    let mut paths: Vec<_> = fs::read_dir("../contracts/")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    //Sort the files by file name
    paths.sort_by_key(|dir| dir.path());

    //For each file, read in the contents and push a YulFile to the yul_files Vec
    for path in paths {
        let contents = fs::read_to_string(path.path())
            .expect("Something went wrong reading from the contracts directory");

        yul_files.push(YulFile {
            file_path: path.path(),
            file_contents: contents,
        });
    }

    //Return the yul files
    yul_files
}
