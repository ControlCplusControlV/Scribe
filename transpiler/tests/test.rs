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
        assert_eq!(*expected_output.first().unwrap(), last_stack_value.as_int());
    }

    #[test]
    fn integration_literal() {
        run_example("add(1,2)".to_string(), vec![], vec![3]);
    }

    #[test]
    fn integration_variables() {
        run_example(
            "
let x := 2
let y := 3
add(x,y)
            "
            .to_string(),
            vec![],
            vec![5],
        );
    }
}
