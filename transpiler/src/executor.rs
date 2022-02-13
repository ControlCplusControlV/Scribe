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
    push.2
    push.0
    push.0
    push.0
    mem.pop.0
    push.3
    push.0
    push.0
    push.0
    mem.pop.1
    mem.push.0
    movup.3
    movup.3
    mem.push.0
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
            .map(|x| format!("{}", x))
            .collect::<Vec<String>>()
            .join("\n")
    );
    assert_eq!(stack.get(0), Some(&BaseElement::new(3)));
    todo!();
}
