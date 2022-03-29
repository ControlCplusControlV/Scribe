use std::collections::{HashMap, HashSet};

use crate::types::*;

use primitive_types::U256;
use zero_machine_code::instructions::*;

//Struct that enables transpilation management. Through implementations, this struct keeps track of the variables,
//open memory addresses, the stack, indentation of Miden assembly and user defined functions.
struct Transpiler {
    local_vars_to_types_and_offsets: HashMap<String, (YulType, u32)>,
    instructions: Vec<Instruction>,
    stack_scratch_space_offset: LocalOffset,

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

#[derive(Default, Clone)]
struct Branch {
    //modified_identifiers: HashSet<TypedIdentifier>,
    //stack_before: Stack,
}

//Struct to represent a stack value.
//Each stack value has an optional typed identifier (ie. x or x:u256) and a YulType
#[derive(Clone, Debug)]
struct StackValue {
    typed_identifier: Option<TypedIdentifier>,
    yul_type: YulType,
}

//Struct to represent the stack, consisting of a Vec of Stack Values
#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

impl Transpiler {
    fn add_instruction(&mut self, inst: Instruction) {
        self.instructions.push(inst);
    }

    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) -> Option<LocalOffset> {
        let saved_local_vars = self.local_vars_to_types_and_offsets;
        self.local_vars_to_types_and_offsets = HashMap::new();
        self.scan_function_definition_for_variables(op);

        for expr in op.block.exprs {
            // TODO
        }

        self.local_vars_to_types_and_offsets = saved_local_vars;
    }

    fn scan_function_definition_for_variables(&mut self, op: &ExprFunctionDefinition) {
        let mut all_variables = HashSet::<(String, YulType)>::new();
        for expr in op.block.exprs {
            match expr {
                Expr::DeclareVariable(e) => {
                    all_variables.extend(e.typed_identifiers.iter().map(|t| (t.identifier, t.yul_type)));
                },
                _ => {}
            }
        }

        let mut counter = 0;
        for (name, yul_type) in all_variables {
            self.local_vars_to_offsets.insert(name, (yul_type, counter));
            counter += 1;
        }
    }

    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) -> Option<LocalOffset> {
        let offsets: Vec<u32> = op.typed_identifiers.iter().map(|iden| self.local_vars_to_offsets.get(&iden.identifier).unwrap().clone()).collect();
        
        match op.rhs {
            Some(rhs) => {
                let rhs = self.transpile_op(op.rhs.deref()).unwrap();

                for offset in offsets {
                    let move_inst = Move32 {
                        val: LocalOrImmediate::Local(rhs),
                        dst: LocalOrImmediate::Local(offset),
                    };
                    self.add_instruction(move_inst);
                }
            }
            None => _
        };
        None
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) -> Option<LocalOffset> {
        let offsets: Vec<u32> = op.identifiers.iter().map(|iden| self.local_vars_to_offsets.get(iden).unwrap().clone()).collect();
        
        let rhs = self.transpile_op(op.rhs.deref()).unwrap();

        for offset in offsets {
            let move_inst = Move32 {
                val: LocalOrImmediate::Local(rhs),
                dst: offset,
            };
            self.add_instruction(Instruction::Move32(move_inst));
        }

        None
    }

    fn transpile_block(&mut self, op: &ExprBlock) -> Option<LocalOffset> {
        for op in &op.exprs {
            self.transpile_op(op);
        }

        None
    }

    fn transpile_literal(&mut self, literal: &ExprLiteral) -> Option<LocalOffset> {
        let stack_scratch = self.stack_scratch_space_offset;

        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            }) => {
                if inferred_type == &Some(YulType::U256) {
                    let offset = self.place_u256_on_stack(*value as u32);
                    Some(offset)
                } else {
                    let offset = self.place_u32_on_stack(*value as u32);
                    Some(offset)
                }
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    fn place_u32_on_stack(&mut self, val: u32) -> LocalOffset {
        let move_inst = Move32 {
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
            let cur = (val % 1u64 << 32) as u32;
            let move_inst = Move32 {
                val: LocalOrImmediate::Immediate(ImmediateOrMacro::Immediate(cur)),
                dst: self.stack_scratch_space_offset,
            };
            self.add_instruction(move_inst);

            self.stack_scratch_space_offset += 1;
        }
        
        to_return
    }

    fn transpile_op(&mut self, expr: &Expr) -> Option<LocalOffset> {
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
            Expr::FunctionDefinition(op) => (),
            // Expr::Break => self.transpile_break(),
            // Expr::Continue => self.transpile_continue(),
            // Expr::Leave => self.transpile_leave(),
            // Expr::Switch(op) => self.transpile_switch(op),
            _ => unreachable!(),
        }
    }
}

//Transpile a Miden program from a Vec of expressions and return the compiled Miden program as a string
pub fn transpile_program(expressions: Vec<Expr>) -> String {
    
}
