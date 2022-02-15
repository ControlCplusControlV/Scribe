use primitive_types::U256;
use scribe::test_utilities::{run_example, MidenResult};

#[ignore]
#[test]
fn u256_add() {
    run_example(
        "
            let x:u256 := 28948022309329048855892746252171976963317496166410141009864396001978282409984
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            u256add(x, y)
        ",
        MidenResult::U256(U256::from_dec_str("50659039041325835497812305941300959685805618291217746767262693003461994217472").unwrap())
    );
}

#[ignore]
#[test]
fn u256_less_than() {
    run_example(
        "
            let x:u256 := 28948022309329048855892746252171976963317496166410141009864396001978282409984
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            let foo:u32 := 0
            if lt(y, x) {
                foo := 1
            }
            foo
        ",
        MidenResult::U32(1)
    );
}

#[ignore]
#[test]
fn u256_match() {
    run_example(
        "
            let x:u256 := 31711016731996786641919559689128982722488122124807605757398297001483711807488
            let foo:u32 := 1;
            switch x {
                case 31711016731996786641919559689128982722488122124807605757398297001483711807488 {
                    foo := 5
                }
            }
            foo
            ",
        MidenResult::U32(5),
    );
}

#[ignore]
#[test]
fn u256_equality() {
    run_example(
        "
            // Not equal
            let x:u256 := 31711016731996786641919559689128982722488122124807605757398297001483711807488
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            let foo:u32 := 0
            if eq(y, x) {
                foo := 1
            }
            foo
        ",
        MidenResult::U32(0)
    );
}

#[ignore]
#[test]
fn u256_function() {
    run_example(
        "
            function add_a_lot(a:u256) -> b {
                let b:u256 = 100;
                if eq(a, 100) {
                    b := add(a, 18446744073709551616)
                } 
            }
            add_a_lot(100);
            ",
        MidenResult::U256(U256::from_dec_str("18446744073709551716").unwrap()),
    );
}

#[ignore]
#[test]
fn u256_sum_odds() {
    run_example(
        "
            let sum_odds:u256 := 0
            let n:u256 := 1218794287928347239847
            for { let i:u256 := 1 } lt(i, n) { i := add(i, 2)} { 
                sum_odds := add(i, sum_odds)
            }
            sum_odds
        ",
        MidenResult::U32(0),
    );
}

#[ignore]
#[test]
fn u256_sqrt() {
    run_example(
        "
        let x:u256 := 100 // Find sqrt of x
        // Start off with z at 1.
        let z:u256 := 1

        // Used below to help find a nearby power of 2.
        let y:u256 := x

        // Find the lowest power of 2 that is at least sqrt(x).
        if iszero(lt(y, 0x100000000000000000000000000000000)) {
            y := shr(128, y) // Like dividing by 2 ** 128.
            z := shl(64, z)
        }
        if iszero(lt(y, 0x10000000000000000)) {
            y := shr(64, y) // Like dividing by 2 ** 64.
            z := shl(32, z)
        }
        if iszero(lt(y, 0x100000000)) {
            y := shr(32, y) // Like dividing by 2 ** 32.
            z := shl(16, z)
        }
        if iszero(lt(y, 0x10000)) {
            y := shr(16, y) // Like dividing by 2 ** 16.
            z := shl(8, z)
        }
        if iszero(lt(y, 0x100)) {
            y := shr(8, y) // Like dividing by 2 ** 8.
            z := shl(4, z)
        }
        if iszero(lt(y, 0x10)) {
            y := shr(4, y) // Like dividing by 2 ** 4.
            z := shl(2, z)
        }
        if iszero(lt(y, 0x8)) {
            // Equivalent to 2 ** z.
            z := shl(1, z)
        }

        // Shifting right by 1 is like dividing by 2.
        z := shr(1, add(z, div(x, z)))
        z := shr(1, add(z, div(x, z)))
        z := shr(1, add(z, div(x, z)))
        z := shr(1, add(z, div(x, z)))
        z := shr(1, add(z, div(x, z)))
        z := shr(1, add(z, div(x, z)))
        z := shr(1, add(z, div(x, z)))

        // Compute a rounded down version of z.
        let zRoundDown:u256 := div(x, z)

        // If zRoundDown is smaller, use it.
        if lt(zRoundDown, z) {
            z := zRoundDown
        }
        ",
        MidenResult::U256(U256::from_dec_str("10").unwrap()),
    );
}
