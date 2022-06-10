use miden_processor::StarkField;
use quickcheck::{Arbitrary, Gen, TestResult};
use quickcheck_macros::quickcheck;
use scribe::{
    executor::execute,
    test_utilities::{miden_to_u256, MidenResult},
    utils::{convert_u256_to_pushes, join_u32s_to_u256, load_all_procs, split_u256_to_u32s},
};

#[derive(Clone, Debug)]
struct U256(primitive_types::U256);

// A reduced range for each u32, to make debugging easier, also tends to find failing cases more
// often because it's vastly more likely that two u32 values will be the same
#[derive(Clone, Debug)]
struct U256Small(primitive_types::U256);

impl Arbitrary for U256 {
    fn arbitrary(g: &mut Gen) -> U256 {
        let bytes = (0..32).map(|_| u8::arbitrary(g)).collect::<Vec<_>>();
        U256(primitive_types::U256::from_little_endian(&bytes))
    }
}

impl Arbitrary for U256Small {
    fn arbitrary(g: &mut Gen) -> U256Small {
        let bytes = (0..32)
            .map(|i| if i % 4 == 0 { u8::arbitrary(g) / 64 } else { 0 })
            .collect::<Vec<_>>();
        U256Small(primitive_types::U256::from_little_endian(&bytes))
    }
}

fn run_miden_function(
    proc: &str,
    stack: Vec<primitive_types::U256>,
    expected: MidenResult,
) -> TestResult {
    let program = format!(
        "use.std::math::u256\n{}\nbegin\n{}\n{}\nend",
        load_all_procs(),
        stack
            .iter()
            .map(convert_u256_to_pushes)
            .collect::<Vec<_>>()
            .join("\n"),
        proc
    );
    println!("{}", program);
    let result = execute(program, vec![]);
    let execution_value = result.unwrap();
    match expected {
        MidenResult::U256(expected) => {
            let output_stack = execution_value.last_stack_state();
            let stack_result = miden_to_u256(execution_value);
            println!("Expected: {}", expected);
            println!("Output  : {}", stack_result);
            println!(
                "O Stack  : {:?}",
                output_stack
                    .iter()
                    .take(8)
                    .map(|x| x.as_int())
                    .collect::<Vec<_>>()
            );
            println!("E Stack  : {:?}", split_u256_to_u32s(&expected));
            TestResult::from_bool(stack_result == expected)
        }
        MidenResult::U32(expected) => {
            let stack_result = execution_value.last_stack_state().first().unwrap().as_int();
            println!("Expected: {}", expected);
            println!("Output  : {}", stack_result);
            TestResult::from_bool(stack_result == expected.into())
        }
    }
}

#[ignore]
#[quickcheck]
fn split_and_join(x: U256) -> TestResult {
    let res = join_u32s_to_u256(split_u256_to_u32s(&x.0));
    TestResult::from_bool(x.0 == res)
}

#[ignore]
#[quickcheck]
fn addition(x: U256, y: U256) -> TestResult {
    let (expected, overflowed) = x.0.overflowing_add(y.0);
    if overflowed {
        return TestResult::discard();
    }
    run_miden_function(
        "exec.u256::add_unsafe",
        vec![x.0, y.0],
        MidenResult::U256(expected),
    )
}

#[quickcheck]
fn multiplication(x: U256, y: U256) -> TestResult {
    let (expected, _overflowed) = (x.0).overflowing_mul(y.0);
    run_miden_function(
        "exec.u256::mul_unsafe",
        vec![(x.0), (y.0)],
        MidenResult::U256(expected),
    )
}

#[ignore]
#[quickcheck]
fn shl(x: U256) -> TestResult {
    let expected = x.0 << 1_u32;
    run_miden_function(
        "exec.u256shl_unsafe",
        vec![x.0],
        MidenResult::U256(expected),
    )
}

#[ignore]
#[quickcheck]
fn less_than(x: U256Small, y: U256Small) -> TestResult {
    let expected = x.0 < y.0;
    run_miden_function(
        "exec.u256lt_unsafe",
        vec![x.0, y.0],
        MidenResult::U32(if expected { 1 } else { 0 }),
    )
}

#[ignore]
#[quickcheck]
fn greater_than(x: U256Small, y: U256Small) -> TestResult {
    let expected = x.0 > y.0;
    run_miden_function(
        "exec.u256gt_unsafe",
        vec![x.0, y.0],
        MidenResult::U32(if expected { 1 } else { 0 }),
    )
}

#[ignore]
#[quickcheck]
fn less_than_or_equal_to(x: U256Small, y: U256Small) -> TestResult {
    let expected = x.0 <= y.0;
    run_miden_function(
        "exec.u256lte_unsafe",
        vec![x.0, y.0],
        MidenResult::U32(if expected { 1 } else { 0 }),
    )
}

#[ignore]
#[quickcheck]
fn greater_than_or_equal_to(x: U256Small, y: U256Small) -> TestResult {
    let expected = x.0 >= y.0;
    run_miden_function(
        "exec.u256gte_unsafe",
        vec![x.0, y.0],
        MidenResult::U32(if expected { 1 } else { 0 }),
    )
}

#[ignore]
#[quickcheck]
fn shr(x: U256) -> TestResult {
    let expected = x.0 >> 1_u32;
    run_miden_function(
        "exec.u256shr_unsafe",
        vec![x.0],
        MidenResult::U256(expected),
    )
}

#[ignore]
#[quickcheck]
fn auto_and(x: U256, y: U256) -> TestResult {
    let expected = x.0 & y.0;
    run_miden_function(
        "exec.u256::and",
        vec![x.0, y.0],
        MidenResult::U256(expected),
    )
}

#[ignore]
#[quickcheck]
fn quickcheck_subtraction(x: U256, y: U256) -> TestResult {
    let (expected, underflowed) = x.0.overflowing_sub(y.0);
    if underflowed {
        return TestResult::discard();
    }
    run_miden_function(
        "exec.u256::sub_unsafe",
        vec![x.0, y.0],
        MidenResult::U256(expected),
    )
}

#[ignore]
#[quickcheck]
fn quickcheck_literals(x: U256) -> TestResult {
    let expected = x.0;
    run_miden_function("", vec![x.0], MidenResult::U256(expected))
}

#[ignore]
#[test]
fn subtraction_with_addition_overflow() {
    let x = join_u32s_to_u256(vec![0, 0, 0, 0, 0, 4, 0, 1]);
    let y = join_u32s_to_u256(vec![0, 0, 0, 0, 0, 0, u32::max_value(), 2]);
    dbg!(x, y);
    let expected = x - y;
    dbg!(expected);
    let test_result = run_miden_function(
        "exec.u256::sub_unsafe",
        vec![x, y],
        MidenResult::U256(expected),
    );
    dbg!(&test_result);
    assert!(!test_result.is_failure());
}

#[test]
fn multiplication_all_limbs() {
    let x = join_u32s_to_u256(vec![8, 7, 6, 5, 4, 3, 2, 1]);
    let y = join_u32s_to_u256(vec![1, 1, 1, 1, 1, 1, 1, 1]);
    dbg!(x, y);
    let (expected, _) = x.overflowing_mul(y);
    dbg!(expected);
    let test_result = run_miden_function(
        "exec.u256::mul_unsafe",
        vec![x, y],
        MidenResult::U256(expected),
    );
    dbg!(&test_result);
    assert!(!test_result.is_failure());
}
