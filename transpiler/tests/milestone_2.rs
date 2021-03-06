use indoc::indoc;
use scribe::test_utilities::compile_example;

#[test]
fn optimization_basic_constant_replacement() {
    compile_example(
        "                                                                                  
            let x:u32 := 10
            x
        ",
        indoc! {"
            begin
                push.10
            end
        "},
    );
}

#[test]
fn optimization_basic_constant_replacement_2() {
    compile_example(
        "                                                                                  
            let x:u32 := 10
            let y:u32 := 5
            add(x, y)
        ",
        indoc! {"
            begin
                push.15
            end
        "},
    );
}

#[test]
fn optimization_unused_var() {
    compile_example(
        "                                                                                  
            let x:u32 := 1
            5
        ",
        indoc! {"
            begin
                push.5
            end
        "},
    );
}

#[test]
fn optimization_last_use() {
    // We'll have to disable constant elimination for this one
    // The point is the `movup` instruction that gets outputted instead of dup, since we don't need
    // to keep a copy on the stack anymore
    compile_example(
        "                                                                                  
            let x:u32 := 1
            let y:u32 := add(x, 2)
            let z:u32 := add(x, 3)
        ",
        indoc! {"
            begin
                push.1
                dup.0
                push.2
                u32add
                movup.1
                push.3
                add
            end
        "},
    );
}

#[test]
fn optimization_let_old_vars_die() {
    // Will probably have to disable some optimizations for this
    //
    // When a var would have been pushed to memory, it should instead be allowed to fall out of the
    // addressable part of the stack, if it's no longer used
    compile_example(
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
        x17
        ",
        indoc! {"
            begin
                push.1
                push.2
                push.3
                push.4
                push.5
                push.6
                push.7
                push.8
                push.9
                push.10
                push.11
                push.12
                push.13
                push.14
                push.15
                push.16
                push.17
            end
        "},
    );
}

#[test]
fn optimization_let_old_vars_die_v2() {
    // Will probably have to disable some optimizations for this
    //
    // When a var would have been pushed to memory, it should instead be allowed to fall out of the
    // addressable part of the stack, if it's no longer used
    compile_example(
        "                                                                                  
        let x1;u256 := 1
        let x2:u256 := 2
        let x3:u256 := 3
        x3 
        ",
        indoc! {"
            begin
            // Assert similarly to the test above to make sure old variables die
            end
        "},
    );
}
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
