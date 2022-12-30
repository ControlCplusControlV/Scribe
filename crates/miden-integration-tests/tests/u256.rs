use crate::utils::{run_example, MidenResult};
use primitive_types::U256;

#[test]
fn u256_hex_literal() {
    run_example(
        "                                                                                  
            let x:u256 := 0x1F6F1604415806848692A606A47
            x
        ",
        MidenResult::U256(U256::from_dec_str("39847239847923879823657234623047").unwrap()),
    );
}

#[test]
fn u256_literal() {
    run_example(
        "
            let x:u256 := 39847239847923879823657234623047
        ",
        MidenResult::U256(U256::from_dec_str("39847239847923879823657234623047").unwrap()),
    );
}

#[test]
fn u256_add() {
    run_example(
        // x = 10 + (20 << 32) + (30 << 64) + (40 << 96) + (50 << 128) + (60 << 160) + (70 << 192) + (80 << 224)
        // y = 1 +  (2 << 32) +  (3 << 64) +  (4 << 96) +  (5 << 128) +  (6 << 160) +  (7 << 192) +  (8 << 224)
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            add(y, x)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "2372475307192593135651930854555638407652639876824448403675529997189131",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_mul() {
    run_example(
        // x = 10 + (20 << 32) + (30 << 64) + (40 << 96) + (50 << 128) + (60 << 160) + (70 << 192) + (80 << 224)
        // y = 1 +  (2 << 32) +  (3 << 64) +  (4 << 96) +  (5 << 128) +  (6 << 160) +  (7 << 192) +  (8 << 224)
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 2
            mul(y, x)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "4313591467622896610276237917373888013913890685135360733955509085798420",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_add_with_carry() {
    run_example(
        // x = (2 ** 32 - 1) + (20 << 32) + (30 << 64) + (40 << 96) + (50 << 128) + (60 << 160) + (70 << 192) + (80 << 224)
        // y = 1 +  (2 << 32) +  (3 << 64) +  (4 << 96) +  (5 << 128) +  (6 << 160) +  (7 << 192) +  (8 << 224)
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977758837866495
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            add(y, x)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "2372475307192593135651930854555638407652639876824448403675534292156416",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_sub() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            sub(x, y)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "1941116160430303474624307062818249606261250808310912330279979088609289",
            )
            .unwrap(),
        ),
    );
}

#[ignore]
#[test]
fn u256_sub_underflow() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            sub(y, x)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "115792087323159981660418150179047860122040009078026900151085826209037651279862",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_and() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            and(x, y)
        ",
        MidenResult::U256(
            U256::from_dec_str("37662610418166091132338348212060737827516158233555356352512")
                .unwrap(),
        ),
    );
}

#[test]
fn u256_or() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            or(x, y)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "2372475307154930525233764763423300059440579138996932245441974640836619",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_xor() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            xor(x, y)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "2372475307117267914815598672290961711228518401169416087208419284484107",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_mixed_types() {
    run_example(
        "
            let x:u256 := 28948022309329048855892746252171976963317496166410141009864396001978282409984
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            let z:u256 := add(x, y)
            let a:u32 := 4
            let b:u32 := 8
            let c := add(a, b)
            z
        ",
        MidenResult::U256(
            U256::from_dec_str("50659039041325835497812305941300959685805618291217746767262693003461994217472")
                .unwrap(),
        ),
    );
}

#[test]
fn u256_stack_overflow() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            let z:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            x
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "2156795733811448305138118958686944006956945342567680366977754542899210",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_less_than() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            let y:u256 := 215679573381144830513811895868694400695694534256768036697775454289921
            lt(x,y)
        ",
        MidenResult::U32(0),
    );
}

#[test]
fn u256_match_no_default() {
    run_example(
        "
            let x:u256 := 31711016731996786641919559689128982722488122124807605757398297001483711807488
            let foo:u32 := 1
            switch x
                case 31711016731996786641919559689128982722488122124807605757398297001483711807488 {
                    foo := 5
                }
            foo
            ",
        MidenResult::U32(5),
    );
}

#[test]
fn u256_match_default_with_match() {
    run_example(
        "
            let x:u256 := 50
            let foo:u32 := 1
            switch x
                case 50 {
                    foo := 5
                }
                default {
                    foo := 83
                }
            foo
            ",
        MidenResult::U32(5),
    );
}

#[test]
fn u256_match_default_no_match() {
    run_example(
        "
            let x:u256 := 80
            let foo:u32 := 1
            switch x
                case 50 {
                    foo := 5
                }
                default {
                    foo := 83
                }
            foo
            ",
        MidenResult::U32(83),
    );
}

#[test]
fn u256_equality() {
    run_example(
        "
            let x:u256 := 31711016731996786641919559689128982722488122124807605757398297001483711807488
            let y:u256 := 21711016731996786641919559689128982722488122124807605757398297001483711807488
            let foo:u32 := 5
            if eq(x, y) {
                foo := 7
            }
            foo
        ",
        MidenResult::U32(5)
    );
}

#[test]
fn u256_function() {
    run_example(
        "
            function add_a_lot(a:u256) -> b:u256 {
                let b:u256 := 100
                if eq(a, 100) {
                    b := add(a, 18446744073709551616)
                } 
            }
            let z:u256 := add_a_lot(100)
            let b := 4
            z
            ",
        MidenResult::U256(U256::from_dec_str("18446744073709551716").unwrap()),
    );
}

#[test]
fn u256_sum_odds() {
    run_example(
        "
            let sum_odds:u256 := 0
            let n:u256 := 72
            for { let i:u256 := 1 } lt(i, n) { i := add(i, 2)} { 
                sum_odds := add(i, sum_odds)
            }
            sum_odds
        ",
        MidenResult::U256(U256::from_dec_str("1296").unwrap()),
    );
}

#[test]
fn u256_shl() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977758837866495
            shl(x)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "4313591467622896610276237917373888013913890685135360733955517675732990",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn u256_shr() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977758837866495
            shr(x)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "1078397866905724152569059479343472003478472671283840183488879418933247",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn mstore_mload_u256() {
    run_example(
        "
            let x:u256 := 2156795733811448305138118958686944006956945342567680366977754542899210
            mstore(100,x)
            mload(100)
        ",
        MidenResult::U256(
            U256::from_dec_str(
                "2156795733811448305138118958686944006956945342567680366977754542899210",
            )
            .unwrap(),
        ),
    );
}

#[test]
fn mstore_mload_u32() {
    run_example(
        "
            let x:u32 := 700
            mstore(100,x)
            mload(100)
        ",
        MidenResult::U32(700),
    );
}

#[test]
fn sum_memory_u32() {
    run_example(
        "
            function sum_from_memory(offset:u32,size:u32) -> b:u32 {
                let b:u32 := 0
                for { let i:u32 := offset } lt(i, add(offset, size)) { i := add(i, 1)} { 
                    b := add(b, mload(i))
                } 
                b
            }
            let x:u32 := 1
            mstore(100,x)
            mstore(101,x)
            mstore(102,x)
            mstore(103,x)
            mstore(104,x)
            sum_from_memory(100, 5)
        ",
        MidenResult::U32(5),
    );
}

#[test]
fn sum_memory_u256() {
    run_example(
        "
            function sum_from_memory(offset:u32,size:u32) -> b:u256 {
                let b:u256 := 0
                for { let i:u32 := offset } lt(i, add(offset, size)) { i := add(i, 1)} { 
                    b := add(b, mload(i))
                } 
                b
            }
            let x:u256 := 1
            mstore(100,x)
            mstore(101,x)
            mstore(102,x)
            mstore(103,x)
            mstore(104,x)
            mstore(105,x)
            let offset:u32 := 100
            let size:u32 := 6
            sum_from_memory(offset, size)
        ",
        MidenResult::U256(U256::from(6)),
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
