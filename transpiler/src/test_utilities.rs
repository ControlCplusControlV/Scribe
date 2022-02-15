// Kind of hacky, maybe move this to a shared crate eventually
use crate::ast_optimization::optimize_ast;
use crate::executor;
use crate::miden_generator;
use crate::parser;
use crate::types::expressions_to_tree;
use colored::*;
use miden_processor::StarkField;

pub enum MidenResult {
    U256(primitive_types::U256),
    U32(u32),
}

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

    print_title("AST");
    println!("{}", expressions_to_tree(&parsed));
    println!();

    let ast = optimize_ast(parsed);
    print_title("Optimized AST");
    println!("{}", expressions_to_tree(&ast));
    println!();

    let miden_code = miden_generator::transpile_program(ast);
    print_title("Generated Miden Assembly");
    println!("{}", miden_code);
    println!();

    let execution_value = executor::execute(miden_code, vec![]).unwrap();
    let stack = execution_value.last_stack_state();
    let last_stack_value = stack.first().unwrap();

    print_title("Miden Output");
    println!("{}", last_stack_value);
    match expected_output {
        MidenResult::U256(expected) => {
            todo!();
        }
        MidenResult::U32(expected) => {
            if expected != last_stack_value.as_int() as u32 {
                print_title("Miden Stack");
                println!("{:?}", stack);
                panic!("Failed, stack result not right");
            }
        }
    }
}
