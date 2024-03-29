use primitive_types::U256;
use crate::utils::{run_example, MidenResult};

#[test]
fn integration_math() {
    run_example("add(1, 2)", MidenResult::U256(U256::from(3)));
    run_example("mul(2, 3)", MidenResult::U256(U256::from(6)));
    run_example("mul(2, 3)", MidenResult::U256(U256::from(6)));
    run_example("sub(4, 2)", MidenResult::U256(U256::from(2)));
    // run_example("div(8, 2)", MidenResult::U256(U256::from(4)));
}

#[test]
fn integration_boolean() {
    run_example("lt(2, 6)", MidenResult::U32(1));
    run_example("lt(6, 2)", MidenResult::U32(0));
    run_example("eq(2, 2)", MidenResult::U32(1));
    run_example("eq(4, 2)", MidenResult::U32(0));
    run_example("or(1, 0)", MidenResult::U256(U256::from(1)));
    run_example("or(0, 0)", MidenResult::U256(U256::from(0)));
    run_example("and(1, 1)", MidenResult::U256(U256::from(1)));
    run_example("and(0, 1)", MidenResult::U256(U256::from(0)));
    run_example("and(0, 0)", MidenResult::U256(U256::from(0)));
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
        MidenResult::U256(U256::from(7)),
    );
}

#[test]
fn integration_if() {
    run_example(
        "
            let x := 2
            let y := 3
            if lt(x, y) {
                x := 5
            }
            x
            ",
        MidenResult::U256(U256::from(5)),
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
        MidenResult::U256(U256::from(378)),
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
        MidenResult::U256(U256::from(5)),
    );
}

#[test]
fn integration_fib() {
    run_example(
        "
            let a:u32 := 0
            let b:u32 := 1
            let c:u32 := 0

            for { let i:u32 := 0 } lt(i, n) { i := add(i, 1)}
            {
                c := add(a,b)
                a := b
                b := c
            }
            b
            ",
        MidenResult::U256(U256::from(89)),
    );
}

#[test]
fn integration_case() {
    run_example(
        "
            let y := 8
            let x := 5
            switch x
                case 3 {
                    y := 5
                }
                case 5 {
                    y := 12
                    let z := 15
                }
                case 8 {
                    y := 15
                }
            y
            ",
        MidenResult::U256(U256::from(12)),
    );
}

#[test]
fn integration_lots_of_vars_u32() {
    run_example(
        "
        let x1:u32 := 1
        let x2:u32 := 2
        let x3:u32 := 3
        let x4:u32 := 4
        let x5:u32 := 5
        let x6:u32 := 6
        let x7:u32 := 7
        let x8:u32 := 8
        let x9:u32 := 9
        let x10:u32 := 10
        let x11:u32 := 11
        let x12:u32 := 12
        let x13:u32 := 13
        let x14:u32 := 14
        let x15:u32 := 15
        let x16:u32 := 16
        let x17:u32 := 17
        let x18:u32 := 18
        x1
            ",
        MidenResult::U32(1),
    );
}

#[test]
fn integration_lots_of_vars() {
    run_example(
        "
        let x1 := 1
        let x2 := 2
        let x3 := 3
        let x4 := 4
        let x5 := 5
        let x6 := 6
        let x7 := 7
        let x8 := 8
        let x9 := 9
        let x10 := 10
        let x11 := 11
        let x12 := 12
        let x13 := 13
        let x14 := 14
        let x15 := 15
        let x16 := 16
        let x17 := 17
        let x18 := 18
        x1
            ",
        MidenResult::U256(U256::from(1)),
    );
}

#[ignore]
#[test]
fn integration_mstore() {
    run_example(
        "
        let x1:u32 := 1
        mstore(42, x1)
        mload(42)
            ",
        MidenResult::U256(U256::from(1)),
    );
}
