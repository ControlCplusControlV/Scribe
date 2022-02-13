use std::collections::{HashMap, HashSet};

use crate::types::*;

#[derive(Default)]
struct Context {
    variables: HashMap<String, u32>,
    indentation: u32,
    next_open_memory_address: u32,
    stack: Stack,
}

type StackValue = HashSet<String>;

#[derive(Default, Clone)]
struct Stack(Vec<StackValue>);

impl std::fmt::Debug for Stack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(for value in self.0.iter() {
            write!(f, "{:?}\n", value);
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

fn declare_var(program: &mut String, op: &ExprDeclareVariable, context: &mut Context) {
    let address = context.next_open_memory_address;
    context.next_open_memory_address += 1;
    context.variables.insert(op.identifier.clone(), address);
    if let Some(rhs) = &op.rhs {
        transpile_op(&rhs, program, context);
        context.stack.top_is_var(&op.identifier);
        // add_line(program, "padw", context);
        // add_line(program, "drop", context);
        // add_line(program, &format!("mem.pop.{}", address), context);
    }
}

fn assignment(program: &mut String, op: &ExprAssignment, context: &mut Context) {
    // TODO: in the future we should be able to just mark that two variables share the same
    // stack address, but I can't quite figure it out for the fibonacci example currently
    if let Expr::Variable(ExprVariableReference {
        identifier: target_ident,
    }) = &*op.rhs
    {
        context
            .stack
            .equate_reference(&op.identifier.clone(), &target_ident);
    } else {
        transpile_op(&op.rhs, program, context);
        context.stack.top_is_var(&op.identifier.clone());
    }
}

fn block(program: &mut String, op: &ExprBlock, context: &mut Context) {
    for op in &op.exprs {
        transpile_op(&op, program, context);
    }
}

fn for_loop(program: &mut String, op: &ExprForLoop, context: &mut Context) {
    block(program, &op.init_block, context);
    let stack_target = context.stack.clone();
    transpile_op(&op.conditional, program, context);
    add_line(program, &format!("while.true"), context);
    // Because the while.true will consume the top of the stack
    context.stack.consume(1);
    context.indentation += 4;
    block(program, &op.interior_block, context);
    block(program, &op.after_block, context);
    add_lines(context.stack.target(stack_target), program, context);
    transpile_op(&op.conditional, program, context);
    // Because the while.true will consume the top of the stack
    context.stack.consume(1);
    context.indentation -= 4;
    add_line(program, &format!("end"), context);
}

fn transpile_repeat(program: &mut String, op: &ExprRepeat, context: &mut Context) {
    let stack_target = context.stack.clone();
    add_line(program, &format!("repeat.{}", op.iterations), context);
    context.indentation += 4;
    block(program, &op.interior_block, context);
    add_lines(context.stack.target(stack_target), program, context);
    context.indentation -= 4;
    add_line(program, &format!("end"), context);
}

fn transpile_miden_function(
    miden_function: &str,
    program: &mut String,
    op: &ExprFunctionCall,
    context: &mut Context,
) {
    for expr in op.exprs.to_vec() {
        transpile_op(&expr, program, context);
    }
    context.stack.consume(2);
    context.stack.add_unknown();
    add_line(program, miden_function, context);
}

fn if_statement(program: &mut String, op: &ExprIfStatement, context: &mut Context) {
    transpile_op(&op.first_expr, program, context);
    add_line(program, &format!("if.true"), context);
    context.indentation += 4;
    block(program, &op.second_expr, context);
    context.indentation -= 4;
    add_line(program, &format!("end"), context);
}

fn insert_literal(program: &mut String, value: u32, context: &mut Context) {
    add_lines(context.stack.push(value), program, context);
}

fn load_variable(program: &mut String, op: &ExprVariableReference, context: &mut Context) {
    add_lines(
        context.stack.push_ref_to_top(&op.identifier),
        program,
        context,
    );
}

fn add_line(program: &mut String, line: &str, context: &Context) {
    *program = format!(
        "{}\n{}{}",
        program,
        " ".repeat(context.indentation.try_into().unwrap()),
        line
    )
}

fn add_lines(lines: Vec<MidenInstruction>, program: &mut String, context: &Context) {
    for line in lines {
        add_line(program, &line, context);
    }
}

fn transpile_op(expr: &Expr, program: &mut String, context: &mut Context) {
    match expr {
        Expr::Literal(value) => insert_literal(program, *value, context),
        Expr::Assignment(op) => assignment(program, op, context),
        Expr::DeclareVariable(op) => declare_var(program, op, context),
        Expr::ForLoop(op) => for_loop(program, op, context),
        Expr::Variable(op) => load_variable(program, op, context),
        Expr::Block(op) => block(program, op, context),
        Expr::IfStatement(op) => if_statement(program, op, context),
        Expr::FunctionCall(op) => {
            // TODO: All functions are assumed to consume 2 stack elements and add one, for now
            // I how Rust handles strings, why are &str and String different? Just to torment me?
            match op.function_name.as_str() {
                // Basic Arithmetic Operations
                "add" => transpile_miden_function("add", program, op, context),
                "sub" => transpile_miden_function("add", program, op, context),
                "mul" => transpile_miden_function("mul", program, op, context),
                "sub" => transpile_miden_function("div", program, op, context),

                // Boolean Operations
                "gt" => transpile_miden_function("gt", program, op, context),
                "lt" => transpile_miden_function("lt", program, op, context),
                "eq" => transpile_miden_function("eq", program, op, context),
                "and" => transpile_miden_function("and", program, op, context),
                "or" => transpile_miden_function("or", program, op, context),

                _ => todo!("Need to implement {} function in miden", op.function_name)

            }
        }
        Expr::Repeat(op) => transpile_repeat(program, op, context),
    }
}

pub fn transpile_program(expressions: Vec<Expr>) -> String {
    let mut context = Context {
        variables: HashMap::new(),
        next_open_memory_address: 0,
        indentation: 4,
        stack: Stack::default(),
    };
    let mut program = "begin".to_string();
    for expr in expressions {
        transpile_op(&expr, &mut program, &mut context);
    }
    context.indentation -= 4;
    add_line(&mut program, "end", &context);
    return program;
}

pub fn optimize_ast(ast: Vec<Expr>) -> Vec<Expr> {
    let mut assignment_visitor = VariableAssignmentVisitor::default();
    let ast = walk_ast(ast, &mut assignment_visitor);
    let const_variables = assignment_visitor.get_const_variables();
    let ast = walk_ast(ast, &mut ConstVariableVisitor { const_variables });
    let ast = walk_ast(ast, &mut ForLoopToRepeatVisitor {});
    ast
}

fn walk_ast<V: ExpressionVisitor>(ast: Vec<Expr>, visitor: &mut V) -> Vec<Expr> {
    let mut new_ast = vec![];
    for expr in ast {
        if let Some(expr) = walk_expr(expr, visitor) {
            new_ast.push(expr);
        }
    }
    return new_ast;
}

trait ExpressionVisitor {
    fn visit_expr(&mut self, expr: Expr) -> Option<Expr>;
}

#[derive(Default)]
struct ConstVariableVisitor {
    const_variables: HashMap<String, u32>,
}

#[derive(Default)]
struct ForLoopToRepeatVisitor {}

#[derive(Default)]
struct VariableAssignmentVisitor {
    assignment_counter: HashMap<String, u32>,
    last_assignment: HashMap<String, u32>,
}

impl VariableAssignmentVisitor {
    fn get_const_variables(&self) -> HashMap<String, u32> {
        self.assignment_counter
            .iter()
            .filter(|(k, v)| **v == 1)
            .filter_map(|(k, _)| {
                if let Some(value) = self.last_assignment.get(k) {
                    return Some((k.clone(), *value));
                }
                return None;
            })
            .collect::<HashMap<String, u32>>()
    }
}

// TODO: unstable for now, as it will incorrectly transform for loops that modify the iterator in
// the interior block. To fix this we should have the variable assignment visitor walk the interior
// block, for assignments. Also need to make sure the var isn't referenced within the for loop
//
// TODO: there's a lot of ways we can miss this optimization currently. Even just flipping i :=
// add(i, 1) to i := add(1, i) will break this optimization. In the future we should support gt,
// subtracting, etc.
impl ExpressionVisitor for ForLoopToRepeatVisitor {
    fn visit_expr(&mut self, expr: Expr) -> Option<Expr> {
        match &expr {
            Expr::ForLoop(ExprForLoop {
                init_block,
                conditional,
                after_block,
                interior_block,
            }) => {
                let start: Option<u32>;
                let iterator_identifier: Option<String>;
                if let Some(first_expr) = (*init_block.exprs).first() {
                    if let Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs }) =
                        first_expr
                    {
                        if let Some(Expr::Literal(value)) = rhs.clone().map(|e| *e) {
                            start = Some(value);
                            iterator_identifier = Some(identifier.to_string());
                        } else {
                            return Some(expr);
                        }
                    } else {
                        return Some(expr);
                    }
                } else {
                    return Some(expr);
                }

                if let Some(Expr::Assignment(assignment)) = (*after_block.exprs).first() {
                    if *assignment
                        == (ExprAssignment {
                            identifier: iterator_identifier.clone().unwrap(),
                            rhs: Box::new(Expr::FunctionCall(ExprFunctionCall {
                                function_name: "add".to_string(),
                                exprs: Box::new(vec![Expr::Variable(ExprVariableReference{
                                    identifier: iterator_identifier.clone().unwrap()}),
                                    Expr::Literal(1), 
                                ]),
                            })),
                        })
                    {}
                } else {
                    return Some(expr);
                }
                if let Expr::FunctionCall(ExprFunctionCall {
                    function_name,
                    exprs ,
                }) = &**conditional
                {
                    if function_name == "lt"
                        && exprs[0]
                            == Expr::Variable(ExprVariableReference {
                                identifier: iterator_identifier.clone().unwrap(),
                            })
                    {
                        if let Expr::Literal(value) = exprs[1] {
                            return Some(Expr::Repeat(ExprRepeat {
                                interior_block: interior_block.clone(),
                                iterations: value - start.unwrap(),
                            }));
                        }
                    }
                } else {
                    return Some(expr);
                }
            }
            _ => {}
        }
        return Some(expr);
    }
}

impl ExpressionVisitor for VariableAssignmentVisitor {
    fn visit_expr(&mut self, expr: Expr) -> Option<Expr> {
        match &expr {
            Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs }) => {
                if let Some(Expr::Literal(v)) = rhs.clone().map(|r| *r) {
                    self.last_assignment.insert(identifier.clone(), v);
                }
                let count = self
                    .assignment_counter
                    .entry(identifier.clone())
                    .or_insert(0);
                *count += 1;
            }
            Expr::Assignment(ExprAssignment { identifier, rhs: _ }) => {
                let count = self
                    .assignment_counter
                    .entry(identifier.clone())
                    .or_insert(0);
                *count += 1;
            }
            _ => {}
        }
        return Some(expr);
    }
}

impl ExpressionVisitor for ConstVariableVisitor {
    fn visit_expr(&mut self, expr: Expr) -> Option<Expr> {
        match &expr {
            Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs: _ }) => {
                if self.const_variables.get(identifier).is_some() {
                    return None;
                }
            }
            Expr::Variable(ExprVariableReference { identifier }) => {
                if let Some(value) = self.const_variables.get(identifier) {
                    return Some(Expr::Literal(*value));
                }
            }
            _ => {}
        }
        return Some(expr);
    }
}

// TODO: it would be nice if there wasn't so much cloning in here
fn walk_expr<V: ExpressionVisitor>(expr: Expr, visitor: &mut V) -> Option<Expr> {
    let expr = visitor.visit_expr(expr.clone());
    if let Some(expr) = expr.clone() {
        return Some(match expr {
            Expr::Literal(x) => expr,
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                exprs,
            }) => Expr::FunctionCall(ExprFunctionCall {
                function_name,
                exprs: Box::new(vec![walk_expr(exprs[0].clone(), visitor).unwrap(), walk_expr(exprs[1].clone(), visitor).unwrap()]),
            }),
            Expr::IfStatement(ExprIfStatement {
                first_expr,
                second_expr,
            }) => Expr::IfStatement(ExprIfStatement {
                first_expr: Box::new(walk_expr(*first_expr, visitor).unwrap()),
                second_expr: Box::new(ExprBlock {
                    exprs: walk_ast(second_expr.exprs, visitor),
                }),
            }),
            Expr::Assignment(ExprAssignment { identifier, rhs }) => {
                Expr::Assignment(ExprAssignment {
                    identifier,
                    rhs: Box::new(walk_expr(*rhs, visitor).unwrap()),
                })
            }
            Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs }) => {
                Expr::DeclareVariable(ExprDeclareVariable {
                    identifier,
                    rhs: rhs.map(|rhs| Box::new(walk_expr(*rhs, visitor).unwrap())),
                })
            }
            Expr::Repeat(ExprRepeat {
                interior_block,
                iterations,
            }) => Expr::Repeat(ExprRepeat {
                iterations,
                interior_block: Box::new(ExprBlock {
                    exprs: walk_ast(interior_block.exprs, visitor),
                }),
            }),
            Expr::ForLoop(ExprForLoop {
                init_block,
                conditional,
                after_block,
                interior_block,
            }) => Expr::ForLoop(ExprForLoop {
                init_block: Box::new(ExprBlock {
                    exprs: walk_ast(init_block.exprs, visitor),
                }),
                conditional: Box::new(walk_expr(*conditional, visitor).unwrap()),
                after_block: Box::new(ExprBlock {
                    exprs: walk_ast(after_block.exprs, visitor),
                }),
                interior_block: Box::new(ExprBlock {
                    exprs: walk_ast(interior_block.exprs, visitor),
                }),
            }),
            Expr::Block(ExprBlock { exprs }) => Expr::Block(ExprBlock {
                exprs: walk_ast(exprs, visitor),
            }),
            Expr::Variable(ExprVariableReference { identifier: _ }) => expr,
        });
    }
    None
}
