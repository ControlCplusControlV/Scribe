use miden::{
    assembly, BaseElement, FieldElement, Program, ProgramInputs, ProofOptions, StarkField,
};

pub fn transpile_yul(yul: &str) -> String {
    return format!(
        "
    begin 
        push.3
        push.5
        add
    end"
    );
}

#[test]
fn test_add() {
    let yul = r#"
        object "fib" {
            code {
               let x := 3; 
               let y := 5;
               let result := add(x, y);
                // TODO: don't know how to return that result in yul
            }
        }
    "#;
    let assembly = transpile_yul(&yul);

    let program = assembly::compile(&assembly).unwrap();

    let options = get_proof_options();
    let pub_inputs = vec![];
    // These are the values in the stack when the program starts
    let inputs = ProgramInputs::from_public(&pub_inputs);
    // Programs return a Vec<u128>, you need to specify how many you expect
    let num_outputs = 1;
    // 8th fibonnaci number
    let expected_result = vec![8];

    let (mut outputs, proof) = miden::execute(&program, &inputs, num_outputs, &options).unwrap();

    assert_eq!(
        expected_result, outputs,
        "Program result was computed incorrectly"
    );

    let verification = miden::verify(*program.hash(), &pub_inputs, &outputs, proof);
    dbg!(&verification);
    assert!(verification.is_ok());
}

#[test]
fn test_fib() {
    let yul = r#"
        object "fib" {
            code {
                let f := 1
                let s := 1
                let next
                for { let i := 0 } lt(i, 10) { i := add(i, 1)}
                {
                if lt(i, 2) {
                    mstore(i, 1)
                }
                if gt(i, 1) {
                    next := add(s, f)
                    f := s
                    s := next
                    mstore(i, s)
                }
                }
            }
        }
    "#;
    let assembly = transpile_yul(&yul);

    let program = assembly::compile(&assembly).unwrap();

    let options = get_proof_options();
    let pub_inputs = vec![1, 0];
    // These are the values in the stack when the program starts
    let inputs = ProgramInputs::from_public(&pub_inputs);
    // Programs return a Vec<u128>, you need to specify how many you expect
    let num_outputs = 1;
    // 8th fibonnaci number
    let expected_result = vec![21];

    let (mut outputs, proof) = miden::execute(&program, &inputs, num_outputs, &options).unwrap();

    assert_eq!(
        expected_result, outputs,
        "Program result was computed incorrectly"
    );

    let verification = miden::verify(*program.hash(), &pub_inputs, &outputs, proof);
    dbg!(&verification);
    assert!(verification.is_ok());
}

fn get_proof_options() -> ProofOptions {
    ProofOptions::new(
        32,
        8,
        0,
        miden::HashFunction::Blake3_256,
        miden::FieldExtension::None,
        8,
        256,
    )
}
