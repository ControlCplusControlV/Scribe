use itertools::Itertools;
use std::collections::{HashMap, HashSet};

use primitive_types::U256;

use crate::{ast_optimization::optimize_ast, types::*};

struct Transpiler {
    variables: HashMap<String, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
    user_functions: HashMap<String, Stack>,
    scoped_identifiers: HashMap<String, TypedIdentifier>,
}

#[derive(Clone, Debug)]
struct StackValue {
    identifier: Option<String>,
    yul_type: YulType,
}

impl From<&TypedIdentifier> for StackValue {
    fn from(typed_identifier: &TypedIdentifier) -> Self {
        StackValue {
            identifier: Some(typed_identifier.identifier.clone()),
            yul_type: typed_identifier.yul_type,
        }
    }
}

#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\n").unwrap();
        for value in self.0.iter() {
            write!(
                f,
                "{}:{}\n",
                value.identifier.clone().unwrap_or("UNKNOWN".to_string()),
                value.yul_type
            )
            .unwrap();
        }
        Ok(())
    }
}

type MidenInstruction = String;

impl Transpiler {
    fn equate_reference(&mut self, x: TypedIdentifier, y: TypedIdentifier) {
        panic!("This no longer works, should fix for optimizations");
        // if x.yul_type != y.yul_type {
        //     panic!("Should never be assigning a {:?} to a {:?}", x, y);
        // }
        // let stack_value = self.stack.0.iter_mut().find(|sv| sv.contains(&y)).unwrap();
        // stack_value.insert(x);
    }

    fn target_stack(&mut self, target_stack: Stack) {
        for v in target_stack.0.iter().rev() {
            // TODO: can do a no-op or padding op if no identifiers
            self.push_identifier_to_top(v.identifier.clone().expect("Need to deal w/ this case"));
        }
        self.stack.0 = self
            .stack
            .0
            .clone()
            .into_iter()
            .take(target_stack.0.len())
            .collect();
    }

    fn add_unknown(&mut self, yul_type: YulType) {
        self.stack.0.insert(
            0,
            StackValue {
                identifier: None,
                yul_type,
            },
        );
    }

    fn push_identifier_to_top(&mut self, identifier: String) {
        let mut offset = 0;
        let stack_value = self
            .stack
            .0
            .iter()
            .find(|sv| {
                if sv.identifier.as_ref() == Some(&identifier) {
                    return true;
                }
                offset += sv.yul_type.stack_width();
                return false;
            })
            .unwrap()
            .clone();
        self.stack.0.insert(0, stack_value.clone());
        self.dup_from_offset(offset, stack_value.yul_type);
    }

    fn dup_from_offset(&mut self, offset: u32, yul_type: YulType) {
        match yul_type {
            YulType::U32 => {
                self.add_line(&format!("dup.{}", offset));
            }
            YulType::U256 => match offset + 7 {
                7 => {
                    self.add_line(&format!("dupw.1"));
                    self.add_line(&format!("dupw.1"));
                }
                11 => {
                    self.add_line(&format!("dupw.2"));
                    self.add_line(&format!("dupw.2"));
                }
                15 => {
                    self.add_line(&format!("dupw.3"));
                    self.add_line(&format!("dupw.3"));
                }
                o => {
                    for _ in (0..8) {
                        self.add_line(&format!("dup.{}", o));
                    }
                }
            },
        };
    }

    fn push(&mut self, value: U256) {
        self.stack.0.insert(
            0,
            StackValue {
                identifier: None,
                yul_type: YulType::U32,
            },
        );
        self.add_line(&format!("push.{}", value));
    }

    fn push_u256(&mut self, value: U256) {
        self.stack.0.insert(
            0,
            StackValue {
                identifier: None,
                yul_type: YulType::U256,
            },
        );
        let mut bytes = [0u8; 32];
        value.to_big_endian(&mut bytes);
        for bytes in &bytes.iter().chunks(4) {
            let mut stack_value: u32 = 0;
            for (i, bytes) in bytes.enumerate() {
                stack_value = stack_value | ((*bytes as u32) << ((3 - i) * 8)) as u32
            }
            self.add_line(&format!("push.{}", stack_value));
        }
    }

    fn _consume_top_stack_values(&mut self, n: u32) {
        for _ in 0..n {
            self.stack.0.remove(0);
        }
    }

    fn dup_top_stack_value(&mut self) {
        let stack_value = self.stack.0.get(0).unwrap().clone();
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("dup");
            }
            YulType::U256 => {
                self.add_line("dupw.1");
                self.add_line("dupw.1");
            }
        }
        self.stack.0.insert(0, stack_value.clone());
    }

    fn drop_top_stack_value(&mut self) {
        let stack_value = self.stack.0.get(0).unwrap().clone();
        self.stack.0.remove(0);
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("drop");
            }
            YulType::U256 => {
                self.add_line("dropw.1");
                self.add_line("dropw.1");
            }
        }
    }

    fn drop_after(&mut self, n: usize) {
        let size_to_keep: u32 = self
            .stack
            .0
            .iter()
            .take(n)
            .map(|sv| sv.yul_type.stack_width())
            .sum();
        let total_size: u32 = self
            .stack
            .0
            .iter()
            .map(|sv| sv.yul_type.stack_width())
            .sum();

        for n in 0..n {
            let stack_length = self.stack.0.len();
            self.stack.0.swap(n, stack_length - 1);
        }
        // TODO: deal w/ when the stack gets too big
        for n in 0..size_to_keep {
            let shift = total_size - 1;
            if shift == 1 {
                self.add_line(&format!("swap"));
            } else {
                self.add_line(&format!("movdn.{}", shift));
            }
        }
        for i in (n..self.stack.0.len()).rev() {
            self.drop_top_stack_value();
        }
    }

    fn top_is_var(&mut self, typed_identifier: TypedIdentifier) {
        let stack_value = self.stack.0.get_mut(0).unwrap();
        stack_value.identifier = Some(typed_identifier.identifier);
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
        self._consume_top_stack_values(1);
        self.indentation += 4;
        self.transpile_block(&op.interior_block);
        self.transpile_block(&op.after_block);
        self.target_stack(stack_target);
        self.transpile_op(&op.conditional);
        self._consume_top_stack_values(1);
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

    fn transpile_switch(&mut self, op: &ExprSwitch) {
        self.add_line("");
        self.transpile_op(&op.expr);
        for case in &op.cases {
            self.transpile_case(&case, &op);
        }
        self.add_line("drop");
        self._consume_top_stack_values(1);
        self.add_line("");
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

        match (
            op.inferred_param_types.first().unwrap(),
            op.function_name.as_ref(),
        ) {
            //u256 operations
            (Some(YulType::U256), "add" | "and" | "or" | "xor" | "iszero" | "eq") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U256);
                return;
            }

            //other operations
            (
                Some(YulType::U32) | None,
                "add" | "sub" | "mul" | "div" | "gt" | "lt" | "eq" | "and" | "or",
            ) => {
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U32);
                self.add_line(op.function_name.as_ref());
                return;
            }

            //iszero
            (Some(YulType::U32) | None, "iszero") => {
                self.add_line("push.0");
                self.add_line("eq");
                self._consume_top_stack_values(1);
                self.add_unknown(YulType::U32);
                return;
            }

            _ => {
                todo!()
            }
        };
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
        self.push_identifier_to_top(typed_identifier.identifier.clone());
    }

    //TODO: stack management not quite working
    fn transpile_function_declaration(&mut self, op: &ExprFunctionDefinition) {
        self.stack = Stack(
            op.params
                .iter()
                .map(|param| StackValue::from(param))
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
            self.push_identifier_to_top(return_ident.clone().identifier);
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
    fn transpile_case(&mut self, op: &ExprCase, switch: &ExprSwitch) {
        self.dup_top_stack_value();
        self.transpile_literal(&op.literal);
        if switch.inferred_type == Some(YulType::U256) {
            // TODO: u256 equality
        } else {
            self.add_line("eq");
            self._consume_top_stack_values(2);
            let stack_target = self.stack.clone();
            self.add_line("if.true");
            self.indentation += 4;
            self.transpile_block(&op.block);
            self.target_stack(stack_target);
            self.indentation -= 4;
            self.add_line("end");
        }
    }

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
            Expr::Switch(op) => self.transpile_switch(op),
            _ => unreachable!(),
        }
    }
    fn add_utility_functions(&mut self) {
        self.add_proc(
            "u256iszero_unsafe",
            r##"
                eq.0
                repeat.7
                    swap
                    eq.0
                    and
                end
            "##,
        );

        self.add_proc(
            "u256add_unsafe",
            r##"
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
            movup.5
            u32addc.unsafe
            movdn.12
            swapw.2
            movup.12
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
        );

        // self.add_proc(
        //     "u256sub_unsafe",
        //     r##"

        //     swapw.3

        //     movup.4

        //     u32.sub.unsafe
        //     #need to check if a<b when subtracting#
        //     if.true
        //         #need to have logic if a<b#

        //     movup.1

        //     movup.4

        //     u32.sub.unsafe

        //     if.true
        //         #need to have logic if a<b#

        //     movup.2

        //     movup.4

        //     u32.sub.unsafe

        //     if.true
        //         #need to have logic if a<b#

        //     #

        //     movup.3

        //     movup.4

        //     u32.sub.unsafe

        //     if.true
        //         #need to have logic if a<b#

        //     swapw.2

        //     #need to handle the last two words#

        //     "##,
        // );

        self.add_proc(
            "u256and_unsafe",
            r##"
            swapw.3
            movup.3
            movup.7
            u32and
            movup.3
            movup.6
            u32and
            movup.3
            movup.5
            u32and
            movup.3
            movup.4
            u32and
            swapw.2
            movup.3
            movup.7
            u32and
            movup.3
            movup.6
            u32and
            movup.3
            movup.5
            u32and
            movup.3
            movup.4
            u32and
            "##,
        );

        self.add_proc(
            "u256or_unsafe",
            r##"
            swapw.3
            movup.3
            movup.7
            u32or
            movup.3
            movup.6
            u32or
            movup.3
            movup.5
            u32or
            movup.3
            movup.4
            u32or
            swapw.2
            movup.3
            movup.7
            u32or
            movup.3
            movup.6
            u32or
            movup.3
            movup.5
            u32or
            movup.3
            movup.4
            u32or
            "##,
        );

        self.add_proc(
            "u256xor_unsafe",
            r##"
            swapw.3
            movup.3
            movup.7
            u32xor
            movup.3
            movup.6
            u32xor
            movup.3
            movup.5
            u32xor
            movup.3
            movup.4
            u32xor
            swapw.2
            movup.3
            movup.7
            u32xor
            movup.3
            movup.6
            u32xor
            movup.3
            movup.5
            u32xor
            movup.3
            movup.4
            u32xor
            "##,
        );

        self.add_proc(
            "u256eq_unsafe",
            r##"
            swapw.3
            eqw
            movdn.8
            dropw
            dropw
            movdn.8
            eqw
            movdn.8
            dropw
            dropw
            and
            "##,
        );
    }
}

pub fn transpile_program(expressions: Vec<Expr>) -> String {
    let mut transpiler = Transpiler {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 0,
        stack: Stack::default(),
        scoped_identifiers: HashMap::new(),
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
