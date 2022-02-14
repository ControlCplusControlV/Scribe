#[cfg(test)]
mod tests {
    use colored::*;
    use miden_processor::StarkField;
    use scribe::ast_optimization::optimize_ast;
    use scribe::executor;
    use scribe::miden_generator;
    use scribe::parser;
    use scribe::types::expressions_to_tree;

    fn run_example(yul_code: &str, expected_output: Vec<u64>) {
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
        if *expected_output.first().unwrap() != last_stack_value.as_int() {
            print_title("Miden Stack");
            println!("{:?}", stack);
            panic!("Failed, stack result not right");
        }
    }

    #[test]
    fn integration_math() {
        run_example("add(1, 2)", vec![3]);
        run_example("mul(2, 3)", vec![6]);
        run_example("mul(2, 3)", vec![6]);
        run_example("sub(4, 2)", vec![2]);
        run_example("div(8, 2)", vec![4]);
    }

    #[test]
    fn integration_boolean() {
        run_example("lt(2, 6)", vec![1]);
        run_example("lt(6, 2)", vec![0]);
        run_example("eq(2, 2)", vec![1]);
        run_example("eq(4, 2)", vec![0]);
        run_example("or(1, 0)", vec![1]);
        run_example("or(0, 0)", vec![0]);
        run_example("and(1, 1)", vec![1]);
        run_example("and(0, 1)", vec![0]);
        run_example("and(0, 0)", vec![0]);
    }

    #[test]
    fn integration_variables() {
        run_example(
            "
            let x := 2
            let y := 3
            x := 4
            add(x, y)
            ",
            vec![7],
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
            ",
            vec![5],
        );
    }

    #[test]
    fn integration_function() {
        run_example(
            "
            function foo() -> b {
                let b := 5
            }
            add(foo(), 1)
            ",
            vec![6],
        );
    }

    #[test]
    fn integration_function_stack_management() {
        run_example(
            "
            function foo() -> b {
                let b := 5
                let a := 3
            }
            mul(foo(), 3)
            ",
            vec![15],
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
            ",
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
            ",
            vec![89],
        );
    }
}
