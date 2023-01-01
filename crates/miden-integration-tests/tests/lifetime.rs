use crate::utils::compile_example;
use indoc::indoc;

#[test]
fn variable_life() {
    // Will probably have to disable some optimizations for this
    //
    // When a var would have been pushed to memory, it should instead be allowed to fall out of the
    // addressable part of the stack, if it's no longer used
    compile_example(
        "
    // Test mstore and mload
    mstore(0x20, 67677686778768)
    let b:u256 := mload(0x20)
    let c:u256 := add(b, 1000)
    mstore(0x20, c)
    let d:u256 = mload(0x20)
    // assert the value of d
    d
        ",
        todo!(),
    );
}

#[test]
fn test_folding() {
    compile_example(
        "
        let a:u256 := add(100, 5)
        let b:u256 := mul(a, 10)
        let c:u256 := div(b, 5)
        let d:u256 := sub(c, 1)
        // Assert this folds into a single PUSH statement
        ",
        todo!(),
    );
}

#[test]
fn test_lifetime() {
    compile_example(
        "
    let a:u256 := 3485488493484388458349458
    let b:u256 := 43589348589349845838993489493
    let c:u256 = add(a, b)
    let d:u256 = add(a, c)
        ",
        todo!(),
    );
}
