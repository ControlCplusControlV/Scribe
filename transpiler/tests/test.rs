#[cfg(test)]
mod tests {
    use colored::*;
    use miden_processor::StarkField;
    use scribe::executor;
    use scribe::miden_generator;
    use scribe::miden_generator::optimize_ast;
    use scribe::parser;
    // use scribe::types;
    use scribe::types::expressions_to_tree;

    fn run_example(yul_code: String, inputs: Vec<u128>, expected_output: Vec<u64>) {
        fn print_title(s: &str) {
            let s1 = format!("=== {} ===", s).blue().bold();
            println!("{}", s1);
            println!(" ");
        }
        println!("");
        println!("");
        print_title("Input Yul");
        println!("{}", yul_code);
        println!("");

        let parsed = parser::parse_yul_syntax(yul_code);

        print_title("AST");
        println!("{}", expressions_to_tree(&parsed));
        println!("");

        let ast = optimize_ast(parsed);
        print_title("Optimized AST");
        println!("{}", expressions_to_tree(&ast));
        println!("");

        let miden_code = miden_generator::transpile_program(ast);
        print_title("Generated Miden Assembly");
        println!("{}", miden_code);
        println!("");

        let execution_value = executor::execute(miden_code, inputs).unwrap();
        let stack = execution_value.last_stack_state();
        let last_stack_value = stack.first().unwrap();

        print_title("Miden Output");
        println!("{}", last_stack_value);
        if *expected_output.first().unwrap() != last_stack_value.as_int() {
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
    fn integration_optimize_const() {
        run_example(
            "
let x := 42
add(x, 2)
            "
            .to_string(),
            vec![],
            vec![5],
        );
    }
    #[test]
    fn integration_variables() {
        run_example(
            "
let x := 2
let y := 3
x := 4
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
if lt(x, y) {
    5
}
            "
            .to_string(),
            vec![],
            vec![5],
        );
    }

    #[test]
    fn integration_for() {
        run_example(
            "
    let x := 2
    for { let i := 0 } lt(i, 5) { i := add(i, 1)} { 
        x := 3
    }
    i
            "
            .to_string(),
            vec![],
            vec![5],
        );
    }

    #[test]
    fn integration_fib() {
        run_example(
            "
            let n := 10
            let a := 0
            let b := 1
            let c := 0

            for { let i := 0 } lt(i, n) { i := add(i, 1)}
            {
                c := add(a,b)
                a := b
                b := c
            }
            b
            "
            .to_string(),
            vec![],
            vec![89],
        );
    }
}
