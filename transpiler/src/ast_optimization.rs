use std::collections::{HashMap, HashSet};

use crate::types::*;

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
                                first_expr: Box::new(Expr::Variable(ExprVariableReference {
                                    identifier: iterator_identifier.clone().unwrap(),
                                })),
                                second_expr: Box::new(Expr::Literal(1)),
                            })),
                        })
                    {}
                } else {
                    return Some(expr);
                }
                if let Expr::FunctionCall(ExprFunctionCall {
                    function_name,
                    first_expr,
                    second_expr,
                }) = &**conditional
                {
                    if function_name == "lt"
                        && *first_expr
                            == Box::new(Expr::Variable(ExprVariableReference {
                                identifier: iterator_identifier.clone().unwrap(),
                            }))
                    {
                        if let Expr::Literal(value) = **second_expr {
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
                first_expr,
                second_expr,
            }) => Expr::FunctionCall(ExprFunctionCall {
                function_name,
                first_expr: Box::new(walk_expr(*first_expr, visitor).unwrap()),
                second_expr: Box::new(walk_expr(*second_expr, visitor).unwrap()),
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
