pub use miden_processor::{Felt as BaseElement, FieldElement, ProgramInputs};

pub fn transpile_yul(yul: &str) -> String {
    return format!(
        "
    begin 
        push.0
        push.0
        push.0

        push.5
        mem.store.0

        push.7
        mem.store.1

        push.0
        push.0
        push.0
        push.0
        mem.load.0
        mem.load.1

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

    let program = miden_assembly::Assembler::new()
        .compile_script(&assembly)
        .unwrap();

    // let options = get_proof_options();
    let pub_inputs = vec![];
    // These are the values in the stack when the program starts
    let inputs = ProgramInputs::new(&pub_inputs, &[], vec![]).unwrap();
    let execution_trace = miden_processor::execute(&program, &inputs).unwrap();
    let stack = execution_trace.last_stack_state();
    dbg!(&stack);
    // stack_init: &[u64],
    // advice_tape: &[u64],
    // advice_sets: Vec<AdviceSet>,
    // Programs return a Vec<u128>, you need to specify how many you expect
    let num_outputs = 1;
    // 8th fibonnaci number
    let expected_result = vec![8];

    //     let (mut outputs, proof) = miden::execute(&program, &inputs, num_outputs, &options).unwrap();
    //
    //     assert_eq!(
    //         expected_result, outputs,
    //         "Program result was computed incorrectly"
    //     );
    //
    //     let verification = miden::verify(*program.hash(), &pub_inputs, &outputs, proof);
    //     dbg!(&verification);
    //     assert!(verification.is_ok());
}

// #[test]
// fn test_fib() {
//     let yul = r#"
//         object "fib" {
//             code {
//                 let f := 1
//                 let s := 1
//                 let next
//                 for { let i := 0 } lt(i, 10) { i := add(i, 1)}
//                 {
//                 if lt(i, 2) {
//                     mstore(i, 1)
//                 }
//                 if gt(i, 1) {
//                     next := add(s, f)
//                     f := s
//                     s := next
//                     mstore(i, s)
//                 }
//                 }
//             }
//         }
//     "#;
//     let assembly = transpile_yul(&yul);
//
//     let program = miden_assembly::compile(&assembly).unwrap();
//
//     let options = get_proof_options();
//     let pub_inputs = vec![1, 0];
//     // These are the values in the stack when the program starts
//     let inputs = ProgramInputs::from_public(&pub_inputs);
//     // Programs return a Vec<u128>, you need to specify how many you expect
//     let num_outputs = 1;
//     // 8th fibonnaci number
//     let expected_result = vec![21];
//
//     let (mut outputs, proof) = miden::execute(&program, &inputs, num_outputs, &options).unwrap();
//
//     assert_eq!(
//         expected_result, outputs,
//         "Program result was computed incorrectly"
//     );
//
//     let verification = miden::verify(*program.hash(), &pub_inputs, &outputs, proof);
//     dbg!(&verification);
//     assert!(verification.is_ok());
// }
//
// fn get_proof_options() -> ProofOptions {
//     ProofOptions::new(
//         32,
//         8,
//         0,
//         miden::HashFunction::Blake3_256,
//         miden::FieldExtension::None,
//         8,
//         256,
//     )
// }
