use papyrus::ast_optimization::optimize_ast;
use papyrus::miden_generator;
use papyrus::parser;
use papyrus::type_inference::infer_types;

use papyrus::types::YulFile;
use std::fs;
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

pub fn write_yul_to_masm(yul_file: YulFile) {
    let parsed = parser::parse_yul_syntax(&yul_file.file_contents);
    let ast = optimize_ast(parsed);
    let ast = infer_types(&ast);

    let miden_code = miden_generator::transpile_program(ast, Default::default());

    fs::write(
        format!(
            "../masm/{}.masm",
            &yul_file.file_path.file_stem().unwrap().to_str().unwrap()
        ),
        miden_code,
    )
    .expect("Unable to write Miden to file.");
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
        }) => (),
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
