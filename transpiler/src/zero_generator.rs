use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

use crate::types::*;

use primitive_types::U256;
use zero_machine_code::instructions::*;

//Struct that enables transpilation management. Through implementations, this struct keeps track of the variables,
//open memory addresses, the stack, indentation of Miden assembly and user defined functions.
struct Transpiler {
    instructions: Vec<GeneralInstruction>,
    //stack_scratch_space_offset: LocalOffset,
    current_stack_frame: StackFrame,
    previous_stack_frames: Vec<StackFrame>,
    /* variables: HashMap<TypedIdentifier, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
    user_functions: HashMap<String, Stack>,
    scoped_identifiers: HashMap<String, TypedIdentifier>,
    branches: VecDeque<Branch>,
    accept_overflow: bool,
    memory_offset: u64,
    procs_used: HashSet<String>, */
}

struct LocalVariables {
    start_address: u32,
    variables: HashMap<TypedIdentifier, u32>,
}

struct EvaluationStack {
    start_address: u32,
    current_offset: u32,
    state: Vec<TypedIdentifier>,
}
struct StackFrame {
    local_variables: LocalVariables,
    evaluation_stack: EvaluationStack,
}

impl EvaluationStack {
    fn offset_of_ith_element(&mut self, i: usize) -> u32 {
        let mut offset = self.start_address;
        for j in 0..i {
            offset += match self.state[j].yul_type {
                YulType::U32 => 1,
                YulType::U256 => 8,
            };
        }
        offset
    }

    fn push(&mut self, id: TypedIdentifier) {
        self.state.push(id)
    }

    fn pop(&mut self) -> Option<TypedIdentifier> {
        self.state.pop()
    }


    fn add3(&mut self) -> Vec<GeneralInstruction> {
        let n = self.state.len();
        assert!(n > 2);

        let x_addr = LocalOrImmediate::Local(self.offset_of_ith_element(n - 2));
        let y_addr = LocalOrImmediate::Local(self.offset_of_ith_element(n - 1));
        let z_addr = LocalOrImmediate::Local(self.offset_of_ith_element(n));

        let dst_addr = self.offset_of_ith_element(n - 2);

        let add3_inst = Instruction::Add3 {
            x: x_addr,
            y: y_addr,
            z: z_addr,
            dst: dst_addr,
        };
        
        vec![GeneralInstruction::Real(add3_inst)]
    }
}


/*#[derive(Default, Clone)]
struct Branch {
    modified_identifiers: HashSet<TypedIdentifier>,
    stack_before: Stack,
}*/

impl Transpiler {
    fn add_instruction(&mut self, inst: GeneralInstruction) {
        self.instructions.push(inst);
    }

    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        self.scan_function_definition_for_variables(op);

        for expr in op.block.exprs.clone() {
            // TODO
        }
    }

    fn scan_function_definition_for_variables(&mut self, op: &ExprFunctionDefinition) {
        let mut current_offset = self.current_stack_frame.local_variables.start_address;
        let mut all_variables = HashSet::<(String, YulType)>::new();
        for expr in op.block.exprs.clone() {
            match expr {
                Expr::DeclareVariable(e) => {
                    all_variables.extend(
                        e.typed_identifiers
                            .iter()
                            .map(|t| (t.identifier.clone(), t.yul_type)),
                    );
                }
                _ => {}
            }
        }

        let mut counter = 0;
        for (name, yul_type) in all_variables {
            self.local_vars_to_types_and_offsets
                .insert(name, (yul_type, counter));
            counter += 1;
        }
    }

    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        // TODO: more than one identifier in variable declaration
        assert_eq!(op.typed_identifiers.len(), 1);
        let identifier = &op.typed_identifiers[0];

        let offset = self
            .local_vars_to_types_and_offsets
            .get(&identifier.identifier)
            .unwrap()
            .clone()
            .1;

        if let Some(rhs) = &op.rhs {
            let rhs = self.transpile_op(rhs.deref());

            for offset in offsets {
                let move_inst = Instruction::Move32 {
                    val: LocalOrImmediate::Local(rhs),
                    dst: offset,
                };
                self.add_instruction(move_inst);
            }
        }
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: more than one identifier in assignment
        assert_eq!(op.identifiers.len(), 1);
        let identifier = &op.identifiers[0];

        let offset = self
            .local_vars_to_types_and_offsets
            .get(identifier)
            .unwrap()
            .clone()
            .1;
        let rhs = self.transpile_op(op.rhs.deref()).unwrap();

        let move_inst = Move32 {
            val: LocalOrImmediate::Local(rhs),
            dst: offset,
        };
        self.add_instruction(Instruction::Move32(move_inst));
    }

    fn transpile_block(&mut self, op: &ExprBlock) {
        for op in &op.exprs {
            self.transpile_op(op);
        }
    }

    fn transpile_literal(&mut self, literal: &ExprLiteral) {
        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            }) => {
                if inferred_type == &Some(YulType::U256) {
                    let offset = self.place_u256_on_stack(*value);
                    Some(offset)
                } else {
                    let offset = self.place_u32_on_stack((*value).try_into().unwrap());
                    Some(offset)
                }
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    fn place_u32_on_stack(&mut self, val: u32) -> LocalOffset {
        let move_inst = Instruction::Move32 {
            val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(val)),
            dst: self.stack_scratch_space_offset,
        };
        self.add_instruction(move_inst);
        let to_return = self.stack_scratch_space_offset;
        self.stack_scratch_space_offset += 1;
        to_return
    }

    fn place_u256_on_stack(&mut self, val: U256) -> LocalOffset {
        let to_return = self.stack_scratch_space_offset;

        for _ in 0..8 {
            let cur: u32 = val.try_into().unwrap();
            let move_inst = Instruction::Move32 {
                val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(cur)),
                dst: self.stack_scratch_space_offset,
            };
            self.add_instruction(move_inst);

            self.stack_scratch_space_offset += 1;
        }

        to_return
    }

    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(value) => self.transpile_literal(value),
            Expr::Assignment(op) => self.transpile_assignment(op),
            Expr::DeclareVariable(op) => self.transpile_variable_declaration(op),
            // Expr::ForLoop(op) => self.transpile_for_loop(op),
            // Expr::Variable(op) => self.transpile_variable_reference(op),
            Expr::Block(op) => self.transpile_block(op),
            // Expr::IfStatement(op) => self.transpile_if_statement(op),
            // Expr::FunctionCall(op) => self.transpile_miden_function(op),
            // Expr::Repeat(op) => self.transpile_repeat(op),
            // We've already compiled the functions
            Expr::FunctionDefinition(op) => todo!(),
            // Expr::Break => self.transpile_break(),
            // Expr::Continue => self.transpile_continue(),
            // Expr::Leave => self.transpile_leave(),
            // Expr::Switch(op) => self.transpile_switch(op),
            _ => unreachable!(),
        }
    }
}

//Transpile a Miden program from a Vec of expressions and return the compiled Miden program as a string
pub fn transpile_program(expressions: Vec<Expr>) -> Vec<Instruction> {
    todo!()
}
