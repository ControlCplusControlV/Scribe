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

#[ignore]
#[test]
fn debug_execution() {
    // You can put a miden program here to debug output, manually modifying it if needed
    let execution_value = execute(
        r##"
        begin
        push.5
        mul.5
        end
        "##
        .to_string(),
        vec![],
    )
    .unwrap();

    println!("Miden Output");
    let stack = execution_value.last_stack_state();
    dbg!(&stack);
    let last_stack_value = stack.first().unwrap();
}
