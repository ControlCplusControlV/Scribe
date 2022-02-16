use primitive_types::U256;
use scribe::test_utilities::{run_example, run_example_temp_u256_mode, MidenResult};
#[test]
fn test_is_zero() {
    run_example_temp_u256_mode(
        "
        let five:u32 := 5
        let x:u32 := iszero(five)
        x
        ",
        MidenResult::U32(0),
    );
}

#[ignore]
#[test]
fn test_is_zero_u256() {
    run_example_temp_u256_mode(
        "
        let five:u256 := 1157923731619542357098500868790785326998466564056403945758400791312963995
        let x:u256 := iszero(five)
        x
        ",
        MidenResult::U32(0),
    );
}
