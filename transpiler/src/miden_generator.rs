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
}

type StackValue = HashSet<String>;

#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for value in self.0.iter() {
            write!(f, "\n{:?}", value).unwrap();
        }
        Ok(())
    }
}

type MidenInstruction = String;

impl Transpiler {
    fn equate_reference(&mut self, x: &str, y: &str) {
        let stack_value = self.stack.0.iter_mut().find(|sv| sv.contains(y)).unwrap();
        stack_value.insert(x.to_string());
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

    fn push_ref_to_top(&mut self, identifier: &str) {
        let mut identifiers = HashSet::new();
        identifiers.insert(identifier.to_string());
        let location = self
            .stack
            .0
            .iter()
            .position(|sv| identifiers.is_subset(sv))
            .unwrap();
        self.stack.0.insert(0, identifiers);
        self.add_line(&format!("dup.{}", location));
    }

    fn push_refs_to_top(&mut self, identifiers: &HashSet<String>) {
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

    fn consume(&mut self, n: u32) {
        for _ in 0..n {
            self.stack.0.remove(0);
        }
    }

    fn drop_after(&mut self, n: usize) {
        let stack_size = self.stack.0.len();
        for n in (0..n) {
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
        println!("Stack after dropping stuff {:?}", self.stack);
    }

    fn top_is_var(&mut self, identifier: &str) {
        let idents = self.stack.0.get_mut(0).unwrap();
        idents.clear();
        idents.insert(identifier.to_string());
    }

    fn add_function_stack(&mut self, function_stack: &Stack) {
        let mut new_stack = Stack::default();
        new_stack.0.append(&mut self.stack.0.clone());
        new_stack.0.append(&mut function_stack.0.clone());
        self.stack = new_stack;
    }
}

impl Transpiler {
    fn transpile_variable_declaration(&mut self, op: &ExprDeclareVariable) {
        let address = self.next_open_memory_address;
        self.next_open_memory_address += 1;
        self.variables.insert(op.identifier.clone(), address);
        if let Some(rhs) = &op.rhs {
            self.transpile_op(rhs);
            self.top_is_var(&op.identifier);
        }
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: in the future we should be able to just mark that two variables share the same
        // stack address, but I can't quite figure it out for the fibonacci example currently
        if let Expr::Variable(ExprVariableReference {
            identifier: target_ident,
        }) = &*op.rhs
        {
            self.equate_reference(&op.identifier.clone(), target_ident);
        } else {
            self.transpile_op(&op.rhs);
            self.top_is_var(&op.identifier.clone());
        }
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
        if let Some(function_stack) = self.user_functions.clone().get(&op.function_name) {
            for expr in op.exprs.clone().into_iter() {
                self.transpile_op(&expr);
            }
            self.add_line(&format!("exec.{}", op.function_name));
            self.add_function_stack(function_stack);
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
        for expr in op.exprs.clone().into_iter() {
            self.transpile_op(&expr);
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
            ExprLiteral::Number(v) => {
                // TODO: check size
                self.push(*v);
            }
            ExprLiteral::String(_) => todo!(),
            &ExprLiteral::Bool(_) => todo!(),
        }
    }

    fn transpile_variable_reference(&mut self, op: &ExprVariableReference) {
        self.push_ref_to_top(&op.identifier);
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
        // let stack_target = self.stack.clone();
        self.add_line(&format!("proc.{}", op.function_name));
        self.indentation += 4;
        self.transpile_block(&op.block);
        // self.target_stack(stack_target);
        for return_ident in &op.returns {
            self.push_ref_to_top(&return_ident);
        }
        self.drop_after(op.returns.len());
        let function_stack = self.stack.clone();
        self.stack = Stack::default();
        self.indentation -= 4;
        self.add_line("end");
        self.user_functions
            .insert(op.function_name.clone(), function_stack);
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
}

pub fn transpile_program(expressions: Vec<Expr>) -> String {
    let mut transpiler = Transpiler {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 0,
        stack: Stack::default(),
        program: "".to_string(),
        user_functions: HashMap::default(),
    };
    let ast = optimize_ast(expressions);
    for expr in &ast {
        match expr {
            Expr::FunctionDefinition(op) => transpiler.transpile_function_declaration(&op),
            _ => (),
        }
    }
    transpiler.add_line("begin");
    transpiler.indentation += 4;
    for expr in ast {
        transpiler.transpile_op(&expr);
    }
    transpiler.indentation -= 4;
    transpiler.add_line("end");
    transpiler.program
}
