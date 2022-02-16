use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use primitive_types::U256;

use crate::{ast_optimization::optimize_ast, types::*};

#[derive(Default)]
struct Transpiler {
    variables: HashMap<String, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
    user_functions: HashMap<String, Stack>,
    scoped_identifiers: HashMap<String, TypedIdentifier>,
}

type StackValue = HashSet<TypedIdentifier>;

#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n").unwrap();
        for value in self.0.iter() {
            if value.is_empty() {
                write!(f, "EMPTY\n").unwrap();
            } else {
                write!(
                    f,
                    "{}\n",
                    value
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
                .unwrap();
            }
        }
        Ok(())
    }
}

type MidenInstruction = String;

impl Transpiler {
    fn equate_reference(&mut self, x: TypedIdentifier, y: TypedIdentifier) {
        if x.yul_type != y.yul_type {
            panic!("Should never be assigning a {:?} to a {:?}", x, y);
        }
        let stack_value = self.stack.0.iter_mut().find(|sv| sv.contains(&y)).unwrap();
        stack_value.insert(x);
    }

    fn target_stack(&mut self, target_stack: Stack) {
        for v in target_stack.0.iter().rev() {
            // TODO: can do a no-op or padding op if no identifiers
            self.push_refs_to_top(v);
        }
    }

    fn add_unknown(&mut self) {
        self.stack.0.insert(0, HashSet::new());
    }

    fn push_ref_to_top(&mut self, variable: TypedIdentifier) {
        let mut variables = HashSet::new();
        variables.insert(variable);
        let location = self
            .stack
            .0
            .iter()
            .position(|sv| variables.is_subset(sv))
            .unwrap();
        self.stack.0.insert(0, variables);
        self.add_line(&format!("dup.{}", location));
    }

    fn push_refs_to_top(&mut self, identifiers: &HashSet<TypedIdentifier>) {
        // TODO: need to figure out what to do when we're targeting a stack that has stack values
        // w/ multiple references, there are probably cases that fail currently, where variables
        // are equal to each other before a for loop but not after
        let location = self
            .stack
            .0
            .iter()
            .position(|sv| identifiers.is_subset(sv))
            .unwrap();
        self.stack.0.insert(0, identifiers.clone());
        self.add_line(&format!("dup.{}", location))
    }

    fn push(&mut self, value: U256) {
        self.stack.0.insert(0, HashSet::new());
        self.add_line(&format!("push.{}", value));
    }

    fn push_u256(&mut self, value: U256) {
        for i in 0..8 {
            self.stack.0.insert(0, HashSet::new());
        }
        let mut bytes = [0u8; 32];
        value.to_big_endian(&mut bytes);
        for bytes in &bytes.iter().chunks(4) {
            let mut stack_value: u32 = 0;
            for (i, bytes) in bytes.enumerate() {
                stack_value = stack_value | ((*bytes as u32) << ((4 - i) * 8)) as u32
            }
            self.add_line(&format!("push.{}", stack_value));
        }
    }

    fn consume(&mut self, n: u32) {
        for _ in 0..n {
            self.stack.0.remove(0);
        }
    }

    fn drop_after(&mut self, n: usize) {
        let stack_size = self.stack.0.len();
        for n in 0..n {
            let shift = stack_size - 1;
            if shift == 1 {
                self.add_line(&format!("swap"));
            } else {
                self.add_line(&format!("movdn.{}", shift));
            }
            self.stack.0.swap(n, stack_size - 1);
        }
        for i in (n..stack_size).rev() {
            self.stack.0.remove(0);
            self.add_line(&format!("drop"));
        }
    }

    fn top_is_var(&mut self, typed_identifier: TypedIdentifier) {
        let idents = self.stack.0.get_mut(0).unwrap();
        idents.clear();
        idents.insert(typed_identifier);
    }

    fn add_function_stack(&mut self, function_stack: &Stack) {
        let mut new_stack = Stack::default();
        new_stack.0.append(&mut self.stack.0.clone());
        new_stack.0.append(&mut function_stack.0.clone());
        self.stack = new_stack;
    }

    fn get_typed_identifier(&self, identifier: &str) -> &TypedIdentifier {
        self.scoped_identifiers
            .get(identifier)
            .expect(&format!("\"{}\" not in scope", identifier))
    }
}

impl Transpiler {
    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        assert_eq!(op.typed_identifiers.len(), 1);
        // let address = self.next_open_memory_address;
        for typed_identifier in &op.typed_identifiers {
            self.scoped_identifiers.insert(
                typed_identifier.identifier.clone(),
                typed_identifier.clone(),
            );
        }
        // self.next_open_memory_address += 1;
        // TODO: multiple declarations not working
        assert_eq!(op.typed_identifiers.len(), 1);
        // TODO: should use memory probably
        // self.variables.insert(op.identifier.clone(), address);
        if let Some(rhs) = &op.rhs {
            self.transpile_op(rhs);
            self.top_is_var(op.typed_identifiers.first().unwrap().clone());
        }
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: more than one identifier in assignment
        assert_eq!(op.identifiers.len(), 1);
        let typed_identifier = self
            .get_typed_identifier(&op.identifiers.first().unwrap())
            .clone();
        // TODO: bring back equating references
        // if let Expr::Variable(ExprVariableReference {
        //     identifier: target_ident,
        // }) = &*op.rhs
        // {
        //     let typed_source_identifier = self.get_typed_identifier(target_ident);
        //     self.equate_reference(typed_identifier.clone(), typed_source_identifier.clone());
        // } else {
        self.transpile_op(&op.rhs);
        self.top_is_var(typed_identifier);
        // }
    }

    fn transpile_block(&mut self, op: &ExprBlock) {
        for op in &op.exprs {
            self.transpile_op(op);
        }
    }

    fn transpile_for_loop(&mut self, op: &ExprForLoop) {
        self.transpile_block(&op.init_block);
        let stack_target = self.stack.clone();
        self.transpile_op(&op.conditional);
        self.add_line("while.true");
        // Because the while.true will consume the top of the stack
        self.consume(1);
        self.indentation += 4;
        self.transpile_block(&op.interior_block);
        self.transpile_block(&op.after_block);
        self.target_stack(stack_target);
        self.transpile_op(&op.conditional);
        self.consume(1);
        self.indentation -= 4;
        self.add_line("end");
    }

    fn transpile_repeat(&mut self, op: &ExprRepeat) {
        let stack_target = self.stack.clone();
        self.add_line(&format!("repeat.{}", op.iterations));
        self.indentation += 4;
        self.transpile_block(&op.interior_block);
        self.target_stack(stack_target);
        self.indentation -= 4;
        self.add_line("end");
    }

    fn transpile_miden_function(&mut self, op: &ExprFunctionCall) {
        for expr in op.exprs.clone().into_iter() {
            self.transpile_op(&expr);
        }
        if let Some(function_stack) = self.user_functions.clone().get(&op.function_name) {
            self.add_line(&format!("exec.{}", op.function_name));
            self.add_function_stack(function_stack);
            return;
        }
        if op.function_name == "iszero" {
            // inline iszero thing
            self.add_line("push.0");
            self.add_line("eq");
            self.consume(1);
            return;
        }
        // TODO: All functions are assumed to consume 2 stack elements and add one, for now
        // I how Rust handles strings, why are &str and String different? Just to torment me?
        let miden_function_name = match op.function_name.as_str() {
            // Basic Arithmetic Operations
            "add" => "add",
            "sub" => "sub",
            "mul" => "mul",
            "div" => "div",

            // Boolean Operations
            "gt" => "gt",
            "lt" => "lt",
            "eq" => "eq",
            "and" => "and",
            "or" => "or",
            // TODO: check whether we've actually generated a function for this call
            _ => todo!("Need to implement {} function in miden", op.function_name),
        };
        // If there is a first param and we've inferred it to be of type u256
        if op.inferred_param_types.first() == Some(&Some(YulType::U256))
            && op.function_name == "add"
        {
            todo!("Need to insert u256 addition here");
        }
        self.consume(2);
        self.add_unknown();
        self.add_line(miden_function_name);
    }

    fn transpile_if_statement(&mut self, op: &ExprIfStatement) {
        self.transpile_op(&op.first_expr);
        self.add_line("if.true");
        self.indentation += 4;
        self.transpile_block(&op.second_expr);
        self.indentation -= 4;
        self.add_line("end");
    }

    fn transpile_literal(&mut self, literal: &ExprLiteral) {
        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            }) => {
                if inferred_type == &Some(YulType::U256) {
                    self.push_u256(*value);
                } else {
                    self.push(*value);
                }
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    fn transpile_variable_reference(&mut self, op: &ExprVariableReference) {
        let typed_identifier = self.get_typed_identifier(&op.identifier);
        self.push_ref_to_top(typed_identifier.clone());
    }

    //TODO: stack management not quite working
    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        self.stack = Stack(
            op.params
                .iter()
                .map(|param| {
                    let mut identifiers = HashSet::new();
                    identifiers.insert(param.clone());
                    identifiers
                })
                .collect(),
        );
        for param in &op.params {
            self.scoped_identifiers
                .insert(param.identifier.clone(), param.clone());
        }
        // let stack_target = self.stack.clone();
        self.add_line(&format!("proc.{}", op.function_name));
        self.indentation += 4;
        self.transpile_block(&op.block);
        // self.target_stack(stack_target);
        for return_ident in &op.returns {
            self.push_ref_to_top(return_ident.clone());
        }
        self.drop_after(op.returns.len());
        let function_stack = self.stack.clone();
        self.stack = Stack::default();
        self.indentation -= 4;
        self.add_line("end");
        self.user_functions
            .insert(op.function_name.clone(), function_stack);
        for param in &op.params {
            self.scoped_identifiers.remove(&param.identifier);
        }
    }

    //TODO: update placeholder
    fn transpile_break(&mut self) {}

    //TODO: update placeholder
    fn transpile_leave(&mut self) {}

    //TODO: update placeholder
    fn transpile_continue(&mut self) {}

    //TODO: update placeholder
    fn transpile_default(&mut self, op: &ExprDefault) {}

    // //TODO: update placeholder
    // fn transpile_case(&mut self, op: &ExprCase) {}

    fn add_line(&mut self, line: &str) {
        self.program = format!(
            "{}\n{}{}",
            self.program,
            " ".repeat(self.indentation.try_into().unwrap()),
            line
        )
    }

    fn add_proc(&mut self, proc_name: &str, lines: &str) {
        self.add_line(&format!("proc.{}", proc_name));
        self.indentation += 4;
        for line in lines.split("\n") {
            self.add_line(&line.trim_start())
        }
        self.indentation -= 4;
        self.add_line("end");
    }

    fn add_lines(&mut self, lines: Vec<MidenInstruction>) {
        for line in lines {
            self.add_line(&line);
        }
    }

    // TODO: re-order AST to have all functions first
    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(value) => self.transpile_literal(value),
            Expr::Assignment(op) => self.transpile_assignment(op),
            Expr::DeclareVariable(op) => self.transpile_variable_declaration(op),
            Expr::ForLoop(op) => self.transpile_for_loop(op),
            Expr::Variable(op) => self.transpile_variable_reference(op),
            Expr::Block(op) => self.transpile_block(op),
            Expr::IfStatement(op) => self.transpile_if_statement(op),
            Expr::FunctionCall(op) => self.transpile_miden_function(op),
            Expr::Repeat(op) => self.transpile_repeat(op),
            // We've already compiled the functions
            Expr::FunctionDefinition(op) => (),
            Expr::Break => self.transpile_break(),
            Expr::Continue => self.transpile_continue(),
            Expr::Leave => self.transpile_leave(),
            Expr::Default(op) => self.transpile_default(op),
            Expr::Case(_) => todo!(),
            // Expr::Case(op) => self.transpile_case(op),
        }
    }
    fn add_utility_functions(&mut self) {
        self.add_proc(
            "u256add",
            r##"
                swapw.2
                swapw.3
                movup.3
                movup.7
                u32add.unsafe
                movup.4
                movup.7
                u32addc.unsafe
                movup.4
                movup.6
                u32addc.unsafe
                movup.4
                movup.7
                u32addc.unsafe
                movdn.4
                swapw.2
                movup.4
                movup.4
                movup.8
                u32addc.unsafe
                movup.4
                movup.7
                u32addc.unsafe
                movup.4
                movup.6
                u32addc.unsafe
                movup.4
                movup.5
                u32addc.unsafe
                drop
            "##,
        )
    }
}

pub fn transpile_program(expressions: Vec<Expr>) -> String {
    let mut transpiler = Transpiler {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 0,
        stack: Stack::default(),
        program: "".to_string(),
        user_functions: HashMap::default(),
        ..Default::default()
    };
    let ast = optimize_ast(expressions);
    for expr in &ast {
        match expr {
            Expr::FunctionDefinition(op) => transpiler.transpile_function_declaration(&op),
            _ => (),
        }
    }
    transpiler.add_utility_functions();
    transpiler.add_line("begin");
    transpiler.indentation += 4;
    for expr in ast {
        transpiler.transpile_op(&expr);
    }
    transpiler.indentation -= 4;
    transpiler.add_line("end");
    transpiler.program
}
