use std::fs;
use miden_processor::StarkField;
use transpiler::executor;
use transpiler::miden_generator;
use transpiler::parser;
use transpiler::types;


#[macro_use]
extern crate pest_derive;

fn main() {

    let yul_contracts = read_yul_contracts();

    for yul_code in yul_contracts{
        let inputs=vec![];
        let parsed = parser::parse_yul_syntax(yul_code.file_contents);

        println!("Parsed Expressions");
        println!("{:?}", parsed);
        println!("");

        let miden_code = miden_generator::transpile_program(parsed);
        println!("Generated Miden Assembly");
        println!("{}", miden_code);
        println!("");

        let execution_value = executor::execute(miden_code, inputs).unwrap();
        let stack = execution_value.last_stack_state();
        let last_stack_value = stack.first().unwrap();

        println!("Miden Output");
        println!("{}", last_stack_value);
    }


}

pub struct YulFile{
    pub file_name: String,
    pub file_contents: String,
}

fn read_yul_contracts() -> Vec<YulFile> {
    let mut yul_files: Vec<YulFile> = Vec::new();

    let file_path = fs::read_dir("../contracts/").unwrap();

    for file in file_path {
        //TODO: get the file name

        let mut unwrapped_file = file.unwrap().path().display().to_string();
        let mut contents = fs::read_to_string(unwrapped_file)
        .expect("Something went wrong readingfrom the contracts directory");
        
        yul_files.push(YulFile{file_name:"".to_string(), file_contents: contents});
    }
return yul_files
}

