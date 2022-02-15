use scribe::test_utilities::run_example;

#[ignore]
#[test]
fn integration_variables() {
    run_example(
        "
            let x:u256 := 2
            let y:u256 := 3
            u256add(x, y)
            ",
        vec![5],
    );
}
