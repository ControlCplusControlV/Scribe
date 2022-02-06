#[cfg(test)]
mod tests {
    use colored::*;
    use miden_processor::StarkField;
    use transpiler::executor;
    use transpiler::miden_generator;
    use transpiler::parser;
    use transpiler::types;

    fn run_example(yul_code: String, inputs: Vec<u128>, expected_output: Vec<u64>) {
        fn print_title(s: &str) {
            let s1 = format!("=== {} ===", s).blue().bold();
            println!("{}", s1);
        }
        println!("");
        println!("");
        print_title("Input Yul");
        println!("{}", yul_code);
        println!("");

        let parsed = parser::parse_yul_syntax(yul_code);

        print_title("Parsed Expressions");
        println!("{:?}", parsed);
        println!("");

        let miden_code = miden_generator::transpile_program(parsed);
        print_title("Generated Miden Assembly");
        println!("{}", miden_code);
        println!("");

        let execution_value = executor::execute(miden_code, inputs).unwrap();
        let stack = execution_value.last_stack_state();
        let last_stack_value = stack.first().unwrap();

        print_title("Miden Output");
        println!("{}", last_stack_value);
        if (*expected_output.first().unwrap() != last_stack_value.as_int()) {
            print_title("Miden Stack");
            println!("{:?}", stack);
            panic!("Failed, stack result not right");
        }
    }

    #[test]
    fn integration_add() {
        run_example("add(1,2)".to_string(), vec![], vec![3]);
    }

    #[test]
    fn integration_variables() {
        run_example(
            "
let x := 2
let y := 3
add(x, y)
            "
            .to_string(),
            vec![],
            vec![5],
        );
    }

    #[test]
    fn integration_if() {
        run_example(
            "
let x := 2
let y := 3
if lt(y, x) {
    5
}
            "
            .to_string(),
            vec![],
            vec![5],
        );
    }
}
