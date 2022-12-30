use crate::utils::{run_example, MidenResult};
use primitive_types::U256;

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

#[test]
fn memory_test() {
    compile_example(
        "
        // populate memory
        mstore(0x20, 1)
        mstore(0x21, 2)
        mstore(0x22, 3)

        // declare variables - we read from memory to make sure constant folding doesn't kick in
        let a:u256 := mload(0x20)
        let b:u256 := mload(0x21)
        let c:u256 := mload(0x22)

        // perform some operations with the variables
        b := add(b, 100)
        c := add(c, a)
        c := add(c, b)
        ",
        todo!(),
    );
}
