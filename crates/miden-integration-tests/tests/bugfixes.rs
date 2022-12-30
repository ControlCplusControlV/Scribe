use crate::utils::{run_example, MidenResult};

#[test]
fn test_is_zero() {
    run_example(
        "
        let five:u32 := 5
        let x:u32 := iszero(five)
        x
        ",
        MidenResult::U32(0),
    );
}

#[test]
fn test_is_zero_u256() {
    run_example(
        "
        let five:u256 := 1157923731619542357098500868790785326998466564056403945758400791312963995
        let x:u256 := iszero(five)
        x
        ",
        MidenResult::U32(0),
    );
}
