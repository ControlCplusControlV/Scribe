extern crate quickcheck_macros;

extern crate insta;
mod repl;
use crate::repl::*;

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

fn main() {
    let opts: Opts = Opts::parse();

    //Match any command line arguments
    match opts.subcmd {
        //If the program is run with "repl" as an argument, start the Miden repl in the terminal
        Some(SubCommand::Repl {
            functions_file,
            stack,
        }) => {
            start_repl(functions_file, stack);
        }
        None => ()
    }
}