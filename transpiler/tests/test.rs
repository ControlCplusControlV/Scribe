use scribe::test_utilities::run_example;

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
            function square(a) -> b {
                let b := mul(a, a)
            }
            function secret() -> c {
                let c := 42
            }
            mul(square(3), secret())
            ",
        vec![378],
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
