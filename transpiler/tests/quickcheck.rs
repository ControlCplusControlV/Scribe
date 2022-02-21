use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use scribe::{executor::execute, test_utilities::miden_to_u256, utils::convert_u256_to_pushes};

#[derive(Clone, Debug)]
struct U256(primitive_types::U256);

impl Arbitrary for U256 {
    fn arbitrary(g: &mut Gen) -> U256 {
        let bytes = (0..32).map(|_| u8::arbitrary(g)).collect::<Vec<_>>();
        U256(primitive_types::U256::from_little_endian(&bytes))
    }
}

fn run_miden_function(proc: &str, stack: Vec<U256>, expected: primitive_types::U256) -> TestResult {
    let program = format!(
        "{}\nbegin\n{}\n{}\nend",
        String::from_utf8(include_bytes!("../src/miden_asm/u256.masm").to_vec()).unwrap(),
        stack
            .iter()
            .map(|sv| convert_u256_to_pushes(&sv.0))
            .collect::<Vec<_>>()
            .join("\n"),
        proc
    );
    println!("{}", program);
    let result = execute(program, vec![]);
    let execution_value = result.unwrap();
    let stack_result = miden_to_u256(execution_value);
    println!("Expected: {}", expected);
    println!("Output  : {}", stack_result);
    // println!("Expected Stack: {}", convert_u256_to_pushes(&expected));
    // println!("Output   Stack: {}", convert_u256_to_pushes(&stack_result));
    TestResult::from_bool(stack_result == expected)
}

#[quickcheck]
fn addition(x: U256, y: U256) -> TestResult {
    let (expected, overflowed) = x.0.overflowing_add(y.0);
    if overflowed {
        return TestResult::discard();
    }
    run_miden_function("exec.u256add_unsafe", vec![x, y], expected)
}

#[quickcheck]
fn auto_and(x: U256, y: U256) -> TestResult {
    let expected = x.0 & y.0;
    run_miden_function("exec.u256and_unsafe", vec![x, y], expected)
}

#[ignore]
#[quickcheck]
fn quickcheck_subtraction(x: U256, y: U256) -> TestResult {
    let (expected, underflowed) = x.0.overflowing_sub(y.0);
    if underflowed {
        return TestResult::discard();
    }
    run_miden_function("exec.u256subc_unsafe", vec![x, y], expected)
}

#[ignore]
#[quickcheck]
fn quickcheck_literals(x: U256) -> TestResult {
    let expected = x.0;
    run_miden_function("", vec![x], expected)
}
