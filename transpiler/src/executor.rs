use miden_processor::{ExecutionError, ExecutionTrace};
pub use miden_processor::{Felt as BaseElement, FieldElement, ProgramInputs};

pub fn execute(program: String, pub_inputs: Vec<u128>) -> Result<ExecutionTrace, ExecutionError> {
    let program = miden_assembly::Assembler::new()
        .compile_script(&program)
        .unwrap();

    // let options = get_proof_options();
    let pub_inputs = vec![];
    // These are the values in the stack when the program starts
    let inputs = ProgramInputs::new(&pub_inputs, &[], vec![]).unwrap();
    miden_processor::execute(&program, &inputs)
}

#[test]
pub fn test_execute() {
    let assembly = "
        begin
        push.1
        push.2
        add
        end
        ";
    let stack = execute(assembly.to_string(), vec![])
        .unwrap()
        .last_stack_state()
        .to_vec();
    assert_eq!(stack.get(0), Some(&BaseElement::new(3)));
}
