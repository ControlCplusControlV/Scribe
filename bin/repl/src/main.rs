extern crate quickcheck_macros;

extern crate insta;
mod repl;
use crate::repl::*;

use clap::Parser;

#[derive(Parser)]
#[clap(version = "1.0", author = "Me")]
struct Opts {
    #[clap(short, long)]
    functions_file: Option<String>,
    #[clap(short, long)]
    stack: Option<String>,
}

fn main() {
    let opts: Opts = Opts::parse();

    start_repl(opts.functions_file, opts.stack);
}
