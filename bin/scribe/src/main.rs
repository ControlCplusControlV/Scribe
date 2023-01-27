use papyrus::miden_generator;
use papyrus::optimizer::optimize_ast;
use papyrus::parser;
use papyrus::type_inference::infer_types;

use papyrus::types::YulFile;
use std::fs;
extern crate quickcheck_macros;

extern crate insta;

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
    let yul_contracts = read_yul_contracts();

    //For each contract in Vec of YulFile
    for yul_code in yul_contracts {
        write_yul_to_masm(yul_code)
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
