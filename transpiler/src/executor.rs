use miden_processor::{ExecutionError, ExecutionTrace};
pub use miden_processor::{Felt as BaseElement, FieldElement, ProgramInputs};

pub fn execute(program: String, _pub_inputs: Vec<u128>) -> Result<ExecutionTrace, ExecutionError> {
    let program = miden_assembly::Assembler::new()
        .compile_script(&program)
        .unwrap();

    let pub_inputs = vec![];
    let inputs = ProgramInputs::new(&pub_inputs, &[], vec![]).unwrap();
    miden_processor::execute(&program, &inputs)
}
