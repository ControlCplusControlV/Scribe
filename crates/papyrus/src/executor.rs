use miden_processor::ExecutionTrace;
pub use miden_processor::{ExecutionError, ProgramInputs};

//Compiles and executes a compiled Miden program, returning the stack and any Miden errors.
//The program is passed in as a String, passed to the Miden Assembler, and then passed into the Miden Processor to be executed
pub fn execute(program: String, _pub_inputs: Vec<u128>) -> Result<ExecutionTrace, MidenError> {
    let program = miden_assembly::Assembler::new(false)
        .compile(&program)
        .map_err(MidenError::AssemblyError)?;

    let pub_inputs = vec![];
    let inputs = ProgramInputs::new(&pub_inputs, &[], vec![]).unwrap();
    miden_processor::execute(&program, &inputs).map_err(MidenError::ExecutionError)
}

//Errors that are returned from the Miden processor during execution.
#[derive(Debug)]
pub enum MidenError {
    AssemblyError(miden_assembly::AssemblyError),
    ExecutionError(ExecutionError),
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
    let _last_stack_value = stack.first().unwrap();
}
