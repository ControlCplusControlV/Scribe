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

#[test]
pub fn test_lt() {
    let assembly = "
begin
push.0
push.0
push.0
push.0
mem.store.0
push.1
mem.store.1
push.2
mem.store.2
push.3
mem.store.3
mem.load.3
mem.load.2
end
        ";
    let stack = execute(assembly.to_string(), vec![])
        .unwrap()
        .last_stack_state()
        .to_vec();
    println!(
        "{}",
        &stack
            .iter()
            .map(|x| format!("{}", x).to_string())
            .collect::<Vec<String>>()
            .join("\n")
    );
    assert_eq!(stack.get(0), Some(&BaseElement::new(3)));
    todo!();
}
