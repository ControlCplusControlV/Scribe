use std::collections::{HashMap, HashSet};

use crate::types::*;

use zero_machine_code::instructions::*;

//Struct that enables transpilation management. Through implementations, this struct keeps track of the variables,
//open memory addresses, the stack, indentation of Miden assembly and user defined functions.
struct Transpiler {
    local_vars_to_offsets: HashMap<String, u32>,
    instructions: Vec<Instruction>,
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
    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        let saved_local_vars = self.local_vars_to_offsets;
        self.local_vars_to_offsets = HashMap::new();
        self.scan_function_definition_for_variables(op);

        for expr in op.block.exprs {
            
        }

        self.local_vars_to_offsets = saved_local_vars;
    }

    fn scan_function_definition_for_variables(&mut self, op: &ExprFunctionDefinition) {
        let mut all_variables = HashSet::<String>::new();
        for expr in op.block.exprs {
            match expr {
                Expr::DeclareVariable(e) => {
                    all_variables.extend(e.typed_identifiers.iter().map(|t| t.identifier));
                },
                _ => {}
            }
        }

        let mut counter = 0;
        for var in all_variables {
            self.local_vars_to_offsets.insert(var, counter);
            counter += 1;
        }
    }

    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        for identifier in op.typed_identifiers {
            match identifier.yul_type {
                YulType::U32 => {

                },
                YulType::U256 => {
                    
                },
            }
        }
        let offsets: Vec<u32> = op.typed_identifiers.iter().map(|iden| self.local_vars_to_offsets.get(&iden.identifier).unwrap().clone()).collect();
        
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        let offsets: Vec<u32> = op.identifiers.iter().map(|iden| self.local_vars_to_offsets.get(iden).unwrap().clone()).collect();
    }

    fn transpile_block(&mut self, op: &ExprBlock) {
        for op in &op.exprs {
            self.transpile_op(op);
        }
    }

    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            // Expr::Literal(value) => self.transpile_literal(value),
            // Expr::Assignment(op) => self.transpile_assignment(op),
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
