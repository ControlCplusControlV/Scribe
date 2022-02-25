// Kind of hacky, maybe move this to a shared crate eventually
use crate::ast_optimization::optimize_ast;
use crate::executor;
use crate::miden_generator;
use crate::parser;
use crate::type_inference::infer_types;
use crate::types::expressions_to_tree;
use colored::*;
use miden_processor::StarkField;
use primitive_types::U256;

pub enum MidenResult {
    U256(primitive_types::U256),
    U32(u32),
}

//Function to display transpile Yul code and display each step of the transpilation process in the terminal.
//This function is only used to demonstrate what Scribe does in a easy to read format.
pub fn run_example(yul_code: &str, expected_output: MidenResult) {
    fn print_title(s: &str) {
        let s1 = format!("=== {} ===", s).blue().bold();
        println!("{}", s1);
        println!(" ");
    }
    println!();
    println!();
    print_title("Input Yul");
    println!("{}", yul_code);
    println!();

    let parsed = parser::parse_yul_syntax(yul_code);

    let ast = optimize_ast(parsed);

    let ast = infer_types(&ast);
    print_title("AST");
    println!("{}", expressions_to_tree(&ast));
    println!();

    let miden_code = miden_generator::transpile_program(ast);
    let mut trimmed_miden_code = miden_code
        .split("\n")
        .skip_while(|line| *line != "# end std lib #")
        .collect::<Vec<_>>()
        .join("\n");
    print_title("Generated Miden Assembly");
    println!("{}", trimmed_miden_code);
    println!();

    let execution_value = executor::execute(miden_code, vec![]).unwrap();
    let stack = execution_value.last_stack_state();
    let last_stack_value = stack.first().unwrap();

    print_title("Miden Output");
    match expected_output {
        MidenResult::U256(expected) => {
            let stack_value = miden_to_u256(execution_value);
            println!("{}", stack_value);
            if expected != stack_value {
                print_title("Miden Stack");
                println!("{:?}", stack);
                panic!("Failed, stack result not right");
            }
        }
        MidenResult::U32(expected) => {
            println!("{}", last_stack_value);
            if expected != last_stack_value.as_int() as u32 {
                print_title("Miden Stack");
                println!("{:?}", stack);
                panic!("Failed, stack result not right");
            }
        }
    }
}

//Converts the top 8 elements on the top of the stack to a U256 struct
//This is used during testing to assert that the Miden output is the correct U256 value
pub fn miden_to_u256(execuiton_trace: miden_processor::ExecutionTrace) -> U256 {
    let u256_bytes = execuiton_trace
        .last_stack_state()
        .iter()
        .take(8)
        .rev()
        .flat_map(|x| {
            let svint = x.as_int() as u32;

            return svint.to_le_bytes();
        })
        .collect::<Vec<_>>();

    U256::from_little_endian(&u256_bytes)
}
