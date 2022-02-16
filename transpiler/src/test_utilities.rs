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
pub fn run_example_temp_u256_mode(yul_code: &str, expected_output: MidenResult) {
    run_example(yul_code, expected_output);
}

fn _run_example(yul_code: &str, expected_output: MidenResult, temp_u256_mode: bool) {
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

    let ast = infer_types(&ast);
    print_title("With type-inference");
    println!("{}", expressions_to_tree(&ast));
    println!();

    let miden_code = miden_generator::transpile_program(ast, temp_u256_mode);
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
            if expected != miden_to_u256(execution_value) {
                print_title("Miden Stack");
                println!("{:?}", stack);
                panic!("Failed, stack result not right");
            }
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

//convert the miden output to U256
pub fn miden_to_u256(execuiton_trace: miden_processor::ExecutionTrace) -> U256 {
    let u256_bytes = execuiton_trace
        .last_stack_state()
        .iter()
        .take(8)
        .flat_map(|x| {
            let svint = x.as_int() as u32;

            return svint.to_le_bytes();
        })
        .collect::<Vec<_>>();

    U256::from_little_endian(&u256_bytes)
}

pub fn run_example(yul_code: &str, expected_output: MidenResult) {
    _run_example(yul_code, expected_output, false);
}
