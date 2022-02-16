// Tests to make sure fixed bugs stay fixed
use scribe::test_utilities::{run_example, MidenResult};
use primitive_types::U256;
#[test]
fn test_is_zero() {
    run_example("
        let five:u32 := 5
        let x:u32 := iszero(five)
        x
        ", MidenResult::U32(0));
    
}

#[test]
fn test_is_zero_u256() {
    run_example("
        let five:u256 := 5
        let x:u256 := iszero(five)
        x
        ", MidenResult::U32(0));
    
}

