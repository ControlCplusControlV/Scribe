use std::collections::{HashMap, HashSet};

use crate::{ast_optimization::optimize_ast, types::*};

#[derive(Default)]
struct Transpiler {
    variables: HashMap<String, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
}

type StackValue = HashSet<String>;

#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(for value in self.0.iter() {
            write!(f, "\n{:?}", value).unwrap();
        })
    }
}

type MidenInstruction = String;

impl Stack {
    // ex if `x := y`, we can keep track of that on our stack
    fn equate_reference(&mut self, x: &str, y: &str) {
        let stack_value = self.0.iter_mut().find(|sv| sv.contains(y)).unwrap();
        stack_value.insert(x.to_string());
    }

    fn target(&mut self, target_stack: Stack) -> Vec<MidenInstruction> {
        let mut instructions = vec![];
        for v in target_stack.0.iter().rev() {
            // TODO: can do a no-op or padding op if no identifiers
            instructions.append(&mut self.push_refs_to_top(v));
        }
        return instructions;
    }

    fn add_unknown(&mut self) {
        self.0.insert(0, HashSet::new());
    }

    fn push_ref_to_top(&mut self, identifier: &str) -> Vec<MidenInstruction> {
        let mut identifiers = HashSet::new();
        identifiers.insert(identifier.to_string());
        let location = self
            .0
            .iter()
            .position(|sv| identifiers.is_subset(sv))
            .unwrap();
        self.0.insert(0, identifiers);
        return vec![format!("dup.{}", location)];
    }

    fn push_refs_to_top(&mut self, identifiers: &HashSet<String>) -> Vec<MidenInstruction> {
        // TODO: need to figure out what to do when we're targeting a stack that has stack values
        // w/ multiple references, there are probably cases that fail currently, where variables
        // are equal to each other before a for loop but not after
        let location = self
            .0
            .iter()
            .position(|sv| identifiers.is_subset(sv))
            .unwrap();
        self.0.insert(0, identifiers.clone());
        return vec![format!("dup.{}", location)];
    }

    fn push(&mut self, value: u32) -> Vec<MidenInstruction> {
        self.0.insert(0, HashSet::new());
        return vec![format!("push.{}", value)];
    }

    fn consume(&mut self, n: u32) {
        for _ in 0..n {
            self.0.remove(0);
        }
    }

    fn top_is_var(&mut self, identifier: &str) {
        let idents = self.0.get_mut(0).unwrap();
        idents.clear();
        idents.insert(identifier.to_string());
    }
}

impl Transpiler {
    fn declare_var(&mut self, op: &ExprDeclareVariable) {
        let address = self.next_open_memory_address;
        self.next_open_memory_address += 1;
        self.variables.insert(op.identifier.clone(), address);
        if let Some(rhs) = &op.rhs {
            self.transpile_op(&rhs);
            self.stack.top_is_var(&op.identifier);
        }
    }

    fn assignment(&mut self, op: &ExprAssignment) {
        // TODO: in the future we should be able to just mark that two variables share the same
        // stack address, but I can't quite figure it out for the fibonacci example currently
        if let Expr::Variable(ExprVariableReference {
            identifier: target_ident,
        }) = &*op.rhs
        {
            self.stack
                .equate_reference(&op.identifier.clone(), &target_ident);
        } else {
            self.transpile_op(&op.rhs);
            self.stack.top_is_var(&op.identifier.clone());
        }
    }

    fn transpile_block(&mut self, op: &ExprBlock) {
        for op in &op.exprs {
            self.transpile_op(&op);
        }
    }

    fn for_loop(&mut self, op: &ExprForLoop) {
        self.transpile_block(&op.init_block);
        let stack_target = self.stack.clone();
        self.transpile_op(&op.conditional);
        self.add_line(&format!("while.true"));
        // Because the while.true will consume the top of the stack
        self.stack.consume(1);
        self.indentation += 4;
        self.transpile_block(&op.interior_block);
        self.transpile_block(&op.after_block);
        let instructions = self.stack.target(stack_target);
        self.add_lines(instructions);
        self.transpile_op(&op.conditional);
        self.stack.consume(1);
        self.indentation -= 4;
        self.add_line(&format!("end"));
    }

    fn transpile_repeat(&mut self, op: &ExprRepeat) {
        let stack_target = self.stack.clone();
        self.add_line(&format!("repeat.{}", op.iterations));
        self.indentation += 4;
        self.transpile_block(&op.interior_block);
        let instructions = self.stack.target(stack_target);
        self.add_lines(instructions);
        self.indentation -= 4;
        self.add_line(&format!("end"));
    }

    fn transpile_miden_function(&mut self, op: &ExprFunctionCall) {
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

            _ => todo!("Need to implement {} function in miden", op.function_name),
        };
        for expr in [&op.first_expr, &op.second_expr] {
            self.transpile_op(expr);
        }
        self.stack.consume(2);
        self.stack.add_unknown();
        self.add_line(miden_function_name);
    }

    fn transpile_if_statement(&mut self, op: &ExprIfStatement) {
        self.transpile_op(&op.first_expr);
        self.add_line(&format!("if.true"));
        self.indentation += 4;
        self.transpile_block(&op.second_expr);
        self.indentation -= 4;
        self.add_line(&format!("end"));
    }

    fn insert_literal(&mut self, value: u32) {
        let instructions = self.stack.push(value);
        self.add_lines(instructions);
    }

    fn load_variable(&mut self, op: &ExprVariableReference) {
        let instructions = self.stack.push_ref_to_top(&op.identifier);
        self.add_lines(instructions);
    }

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

    fn transpile_op(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(value) => self.insert_literal(*value),
            Expr::Assignment(op) => self.assignment(op),
            Expr::DeclareVariable(op) => self.declare_var(op),
            Expr::ForLoop(op) => self.for_loop(op),
            Expr::Variable(op) => self.load_variable(op),
            Expr::Block(op) => self.transpile_block(op),
            Expr::IfStatement(op) => self.transpile_if_statement(op),
            Expr::FunctionCall(op) => self.transpile_miden_function(op),
            Expr::Repeat(op) => self.transpile_repeat(op),
        }
    }
}

pub fn transpile_program(expressions: Vec<Expr>) -> String {
    let mut transpiler = Transpiler {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 4,
        stack: Stack::default(),
        program: "begin".to_string(),
    };
    let ast = optimize_ast(expressions);
    for expr in ast {
        transpiler.transpile_op(&expr);
    }
    transpiler.indentation -= 4;
    transpiler.add_line("end");
    return transpiler.program;
}
