// Tests to make sure fixed bugs stay fixed
use scribe::test_utilities::{run_example, MidenResult};

#[test]
fn test_is_zero() {
    run_example("
        code {
        let five:u32 := 5
        let x:u32 := iszero(five)
        x
        }", MidenResult::U32(10));
    
}

  
