use itertools::Itertools;
use std::collections::{HashMap, HashSet, VecDeque};

use primitive_types::U256;

use crate::{ast_optimization::optimize_ast, types::*, utils::convert_u256_to_pushes};

struct Transpiler {
    variables: HashMap<TypedIdentifier, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
    program: String,
    user_functions: HashMap<String, Stack>,
    scoped_identifiers: HashMap<String, TypedIdentifier>,
    branches: VecDeque<Branch>,
}

#[derive(Default, Clone)]
struct Branch {
    modified_identifiers: HashSet<TypedIdentifier>,
    stack_before: Stack,
}

#[derive(Clone, Debug)]
struct StackValue {
    typed_identifier: Option<TypedIdentifier>,
    yul_type: YulType,
}

impl From<&TypedIdentifier> for StackValue {
    fn from(typed_identifier: &TypedIdentifier) -> Self {
        StackValue {
            typed_identifier: Some(typed_identifier.clone()),
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
                value
                    .typed_identifier
                    .clone()
                    .map(|ti| ti.identifier)
                    .unwrap_or("UNKNOWN".to_string()),
                value.yul_type
            )
            .unwrap();
        }
        Ok(())
    }
}

impl Transpiler {
    fn equate_reference(&mut self, x: TypedIdentifier, y: TypedIdentifier) {
        panic!("This no longer works, should fix for optimizations");
        // if x.yul_type != y.yul_type {
        //     panic!("Should never be assigning a {:?} to a {:?}", x, y);
        // }
        // let stack_value = self.stack.0.iter_mut().find(|sv| sv.contains(&y)).unwrap();
        // stack_value.insert(x);
    }

    fn indent(&mut self) {
        self.indentation += 4;
    }
    fn outdent(&mut self) {
        self.indentation -= 4;
    }

    fn begin_branch(&mut self) {
        self.branches.push_front(Branch {
            stack_before: self.stack.clone(),
            ..Default::default()
        });
    }

    fn end_branch(&mut self) {
        let branch = self.branches.pop_front().unwrap();
        self.add_comment("cleaning up after branch");
        self.indent();
        for modified_identifier in branch.modified_identifiers {
            if branch
                .stack_before
                .0
                .iter()
                .find(|sv| sv.typed_identifier == Some(modified_identifier.clone()))
                .is_none()
            {
                self.update_identifier_in_memory(modified_identifier)
            }
        }
        self.target_stack(branch.stack_before);
        self.outdent();
    }

    fn target_stack(&mut self, target_stack: Stack) {
        for v in target_stack.0.iter().rev() {
            // TODO: can do a no-op or padding op if no identifiers
            self.push_identifier_to_top(
                v.typed_identifier
                    .clone()
                    .expect("Need to deal w/ this case"),
            );
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
                typed_identifier: None,
                yul_type,
            },
        );
    }

    fn update_identifier_in_memory(&mut self, typed_identifier: TypedIdentifier) {
        self.push_identifier_to_top(typed_identifier);
        self.pop_top_var_to_memory();
    }

    fn load_identifier_from_memory(&mut self, typed_identifier: TypedIdentifier) {
        self.add_comment(&format!(
            "push {} from memory to top of stack",
            typed_identifier.identifier
        ));
        let address = self.variables.get(&typed_identifier).cloned().unwrap();
        match typed_identifier.yul_type {
            YulType::U32 => {
                self.add_line(&format!("mem.push.{}", address));
                self.add_line("dup");
                self.add_line("dropw");
            }
            YulType::U256 => {
                self.add_line(&format!("mem.push.{}", address + 1));
                self.add_line(&format!("mem.push.{}", address + 0));
            }
        }
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: Some(typed_identifier.clone()),
                yul_type: typed_identifier.yul_type,
            },
        );
        self.newline();
    }

    fn push_identifier_to_top(&mut self, typed_identifier: TypedIdentifier) {
        let mut offset = 0;
        self.prepare_for_stack_values(&typed_identifier.yul_type);
        let stack_value = self
            .stack
            .0
            .iter()
            .find(|sv| {
                if sv.typed_identifier.as_ref() == Some(&typed_identifier) {
                    return true;
                }
                offset += sv.yul_type.stack_width();
                return false;
            })
            .cloned();
        match stack_value {
            Some(stack_value) => {
                self.stack.0.insert(
                    0,
                    StackValue {
                        typed_identifier: Some(typed_identifier.clone()),
                        yul_type: stack_value.yul_type,
                    },
                );
                self.add_comment(&format!(
                    "pushing {} to the top",
                    typed_identifier.identifier
                ));
                self.indent();
                self.dup_from_offset(offset, stack_value.yul_type);
                self.outdent()
            }
            None => {
                self.load_identifier_from_memory(typed_identifier);
            }
        }
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
        self.prepare_for_stack_values(&YulType::U32);
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: None,
                yul_type: YulType::U32,
            },
        );
        self.add_comment(&format!("u32 literal {}", value));
        self.add_line(&format!("push.{}", value));
        self.newline();
    }

    fn prepare_for_stack_values(&mut self, yul_type: &YulType) {
        while self.get_size_of_stack() + yul_type.stack_width() > 16 {
            self.add_comment(&format!(
                "stack would be too large after {}, popping to memory",
                yul_type,
            ));
            // dbg!(&self.stack);
            self.indent();
            self.pop_bottom_var_to_memory();
            self.outdent();
            // dbg!(&self.stack);
        }
    }

    fn pop_bottom_var_to_memory(&mut self) {
        let stack_value = self.stack.0.last().cloned().unwrap();

        let address = match self
            .variables
            .get(&stack_value.typed_identifier.clone().unwrap())
        {
            Some(address) => *address,
            None => {
                let address = self.next_open_memory_address;
                self.variables
                    .insert(stack_value.typed_identifier.clone().unwrap(), address);
                self.next_open_memory_address += if stack_value.yul_type == YulType::U256 {
                    2
                } else {
                    1
                };
                address
            }
        };
        let stack_above: Vec<StackValue> = self
            .stack
            .0
            .clone()
            .into_iter()
            .take(self.stack.0.len() - 1)
            .collect();
        let num_stack_values_above: u32 =
            stack_above.iter().map(|sv| sv.yul_type.stack_width()).sum();
        self.add_comment(&format!(
            "Moving {} to top of stack",
            stack_value.typed_identifier.as_ref().unwrap().identifier
        ));
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line(&format!("movup.{}", num_stack_values_above));
            }
            YulType::U256 => {
                match num_stack_values_above {
                    1 => {
                        self.add_line(&format!("movdn.8"));
                    }
                    8 => {
                        self.add_line(&format!("movupw.3"));
                        self.add_line(&format!("movupw.3"));
                    }
                    o => {
                        for _ in (0..8) {
                            self.add_line(&format!("movup.{}", o + 7));
                        }
                    }
                };
            }
        }
        self.stack.0.pop();
        self.stack.0.insert(0, stack_value.clone());
        self.pop_top_var_to_memory();
    }

    fn pop_top_var_to_memory(&mut self) {
        let stack_value = self.stack.0.first().unwrap().clone();

        let address = match self
            .variables
            .get(&stack_value.typed_identifier.clone().unwrap())
        {
            Some(address) => *address,
            None => {
                let address = self.next_open_memory_address;
                self.variables
                    .insert(stack_value.typed_identifier.clone().unwrap(), address);
                self.next_open_memory_address += if stack_value.yul_type == YulType::U256 {
                    2
                } else {
                    1
                };
                address
            }
        };
        self.add_comment(&format!(
            "popping {} from top of stack to memory",
            stack_value.typed_identifier.clone().unwrap()
        ));
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("padw");
                self.add_line("drop");
                self.add_line(&format!("mem.pop.{}", address));
            }
            YulType::U256 => {
                self.add_line(&format!("mem.pop.{}", address));
                self.add_line(&format!("mem.pop.{}", address + 1));
            }
        }
        self.newline();
        self.stack.0.remove(0);
    }

    fn get_size_of_stack(&self) -> u32 {
        self.stack
            .0
            .iter()
            .map(|sv| sv.yul_type.stack_width())
            .sum()
    }

    fn push_u256(&mut self, value: U256) {
        self.prepare_for_stack_values(&YulType::U256);
        self.add_comment(&format!("u256 literal: {}", value));
        self.stack.0.insert(
            0,
            StackValue {
                typed_identifier: None,
                yul_type: YulType::U256,
            },
        );
        self.add_line(&convert_u256_to_pushes(&value));
        self.newline();
    }

    fn _consume_top_stack_values(&mut self, n: u32) {
        for _ in 0..n {
            self.stack.0.remove(0);
        }
    }

    fn dup_top_stack_value(&mut self) {
        self.add_comment(&format!("duping top stack value"));
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
        self.newline();
    }

    fn drop_top_stack_value(&mut self) {
        self.add_comment(&format!("dropping top stack value"));
        let stack_value = self.stack.0.get(0).unwrap().clone();
        self.stack.0.remove(0);
        match stack_value.yul_type {
            YulType::U32 => {
                self.add_line("drop");
            }
            YulType::U256 => {
                self.add_line("dropw");
                self.add_line("dropw");
            }
        }
        self.newline();
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
        self.add_comment(&format!("dropping after {} ", n));
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
        self.newline();
    }

    fn top_is_var(&mut self, typed_identifier: TypedIdentifier) {
        let stack_value = self.stack.0.get_mut(0).unwrap();
        stack_value.typed_identifier = Some(typed_identifier);
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
        self.add_comment(&format!(
            "Assigning to {}",
            op.typed_identifiers.first().unwrap().identifier
        ));
        self.indent();
        if let Some(rhs) = &op.rhs {
            self.transpile_op(rhs);
            self.top_is_var(op.typed_identifiers.first().unwrap().clone());
        }
        self.outdent();
    }

    fn transpile_assignment(&mut self, op: &ExprAssignment) {
        // TODO: more than one identifier in assignment
        assert_eq!(op.identifiers.len(), 1);
        let typed_identifier = self
            .get_typed_identifier(&op.identifiers.first().unwrap())
            .clone();
        self.add_comment(&format!("Assigning to {}", typed_identifier.identifier));
        if let Some(branch) = self.branches.front_mut() {
            branch.modified_identifiers.insert(typed_identifier.clone());
        }
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
        self.add_comment("-- conditional --");
        self.transpile_op(&op.conditional);
        self.add_line("while.true");
        // Because the while.true will consume the top of the stack
        self._consume_top_stack_values(1);
        self.indent();
        self.begin_branch();

        self.add_comment("-- interior block --");
        self.indent();
        self.transpile_block(&op.interior_block);
        self.outdent();
        self.newline();

        self.add_comment("-- after block --");
        self.indent();
        self.transpile_block(&op.after_block);
        self.newline();
        self.end_branch();
        self.outdent();
        self.newline();

        self.add_comment("-- conditional --");
        self.indent();
        self.transpile_op(&op.conditional);
        self.outdent();
        self.newline();
        self._consume_top_stack_values(1);
        self.outdent();

        self.add_line("end");
        self.newline();
    }

    fn transpile_repeat(&mut self, op: &ExprRepeat) {
        let stack_target = self.stack.clone();
        self.add_line(&format!("repeat.{}", op.iterations));
        self.indent();
        self.transpile_block(&op.interior_block);
        self.target_stack(stack_target);
        self.outdent();
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

        self.add_comment(&format!("{}()", op.function_name));

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
            (Some(YulType::U256), "add" | "sub" | "and" | "or" | "xor") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U256);
                return;
            }

            (Some(YulType::U256), "iszero" | "eq" | "lt") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(2);
                self.add_unknown(YulType::U32);
                return;
            }
            (Some(YulType::U256), "shl" | "shr") => {
                let u256_operation = format!("exec.u256{}_unsafe", op.function_name.as_str());
                self.add_line(&u256_operation);
                self._consume_top_stack_values(1);
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
        self.newline();
    }

    fn transpile_if_statement(&mut self, op: &ExprIfStatement) {
        self.transpile_op(&op.first_expr);
        self._consume_top_stack_values(1);
        self.add_line("if.true");
        self.indent();
        self.begin_branch();
        self.transpile_block(&op.second_expr);
        self.end_branch();
        self.outdent();
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
        self.push_identifier_to_top(typed_identifier.clone());
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
        self.indent();
        self.transpile_block(&op.block);
        // self.target_stack(stack_target);
        for return_ident in &op.returns {
            self.push_identifier_to_top(return_ident.clone());
        }
        self.drop_after(op.returns.len());
        let function_stack = self.stack.clone();
        self.stack = Stack::default();
        self.outdent();
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
        self.prepare_for_stack_values(&switch.inferred_type.unwrap());
        self.dup_top_stack_value();
        self.transpile_literal(&op.literal);
        if switch.inferred_type == Some(YulType::U256) {
            self.add_line("exec.u256eq_unsafe");
        } else {
            self.add_line("eq");
        }
        self.add_unknown(YulType::U32);
        self._consume_top_stack_values(2);
        self.begin_branch();
        self._consume_top_stack_values(1);
        self.add_line("if.true");
        self.indent();
        self.transpile_block(&op.block);
        self.end_branch();
        self.outdent();
        self.add_line("end");
    }

    fn add_line(&mut self, line: &str) {
        self.program = format!(
            "{}\n{}{}",
            self.program,
            " ".repeat(self.indentation.try_into().unwrap()),
            line
        )
    }

    fn newline(&mut self) {
        self.program = format!("{}\n", self.program)
    }

    fn add_comment(&mut self, comment: &str) {
        // dbg!(comment);
        self.program = format!(
            "{}\n{}# {} #",
            self.program,
            " ".repeat(self.indentation.try_into().unwrap()),
            comment
        )
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
        let bytes = include_bytes!("./miden_asm/u256.masm");
        let procs = String::from_utf8(bytes.to_vec()).unwrap();
        self.add_line(&format!("{}\n", procs));
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
        branches: VecDeque::new(),
    };
    let ast = optimize_ast(expressions);
    transpiler.add_utility_functions();
    transpiler.add_line("# end std lib #");
    for expr in &ast {
        match expr {
            Expr::FunctionDefinition(op) => transpiler.transpile_function_declaration(&op),
            _ => (),
        }
    }
    transpiler.add_line("begin");
    transpiler.indent();
    for expr in ast {
        transpiler.transpile_op(&expr);
    }
    transpiler.outdent();
    transpiler.add_line("end");
    transpiler.program
}
