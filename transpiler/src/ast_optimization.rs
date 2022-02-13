use std::{collections::HashMap, vec};

use crate::types::*;

pub fn optimize_ast(ast: Vec<Expr>) -> Vec<Expr> {
    let mut assignment_visitor = VariableAssignmentVisitor::default();
    let ast = walk_ast(ast, &mut assignment_visitor);
    let const_variables = assignment_visitor.get_const_variables();
    let ast = walk_ast(ast, &mut ConstVariableVisitor { const_variables });

    walk_ast(ast, &mut ForLoopToRepeatVisitor {})
}

fn walk_ast<V: ExpressionVisitor>(ast: Vec<Expr>, visitor: &mut V) -> Vec<Expr> {
    let mut new_ast = vec![];
    for expr in ast {
        if let Some(expr) = walk_expr(expr, visitor) {
            new_ast.push(expr);
        }
    }
    new_ast
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
            .filter(|(_k, v)| **v == 1)
            .filter_map(|(k, _)| {
                if let Some(value) = self.last_assignment.get(k) {
                    return Some((k.clone(), *value));
                }
                None
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
                                exprs: Box::new(vec![
                                    Expr::Variable(ExprVariableReference {
                                        identifier: iterator_identifier.clone().unwrap(),
                                    }),
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
                    exprs,
                }) = &**conditional
                {
                    if function_name == "lt"
                        && exprs[0]
                            == Expr::Variable(ExprVariableReference {
                                identifier: iterator_identifier.unwrap(),
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
        Some(expr)
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
        Some(expr)
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
        Some(expr)
    }
}

// TODO: it would be nice if there wasn't so much cloning in here
fn walk_expr<V: ExpressionVisitor>(expr: Expr, visitor: &mut V) -> Option<Expr> {
    let expr = visitor.visit_expr(expr);
    if let Some(expr) = expr {
        return Some(match expr {
            //Expr is literal
            Expr::Literal(_x) => expr,

            //Expr is function call
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                exprs,
            }) => Expr::FunctionCall(ExprFunctionCall {
                function_name,
                exprs: Box::new(vec![
                    walk_expr(exprs[0].clone(), visitor).unwrap(),
                    walk_expr(exprs[1].clone(), visitor).unwrap(),
                ]),
            }),

            //Expr is if statement
            Expr::IfStatement(ExprIfStatement {
                first_expr,
                second_expr,
            }) => Expr::IfStatement(ExprIfStatement {
                first_expr: Box::new(walk_expr(*first_expr, visitor).unwrap()),
                second_expr: Box::new(ExprBlock {
                    exprs: walk_ast(second_expr.exprs, visitor),
                }),
            }),

            //Expr is assignment
            Expr::Assignment(ExprAssignment { identifier, rhs }) => {
                Expr::Assignment(ExprAssignment {
                    identifier,
                    rhs: Box::new(walk_expr(*rhs, visitor).unwrap()),
                })
            }

            //Expr is declare variable
            Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs }) => {
                Expr::DeclareVariable(ExprDeclareVariable {
                    identifier,
                    rhs: rhs.map(|rhs| Box::new(walk_expr(*rhs, visitor).unwrap())),
                })
            }

            //TODO: Expr is function definition
            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name,
                typed_identifier_list,
                return_typed_identifier_list,
                block,
            }) => Expr::Literal(0),

            //Expr is repeat
            Expr::Repeat(ExprRepeat {
                interior_block,
                iterations,
            }) => Expr::Repeat(ExprRepeat {
                iterations,
                interior_block: Box::new(ExprBlock {
                    exprs: walk_ast(interior_block.exprs, visitor),
                }),
            }),

            //Expr is for loop
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

            //Expr is block
            Expr::Block(ExprBlock { exprs }) => Expr::Block(ExprBlock {
                exprs: walk_ast(exprs, visitor),
            }),

            //Expr is variable
            Expr::Variable(ExprVariableReference { identifier: _ }) => expr,
        });
    }
    None
}
