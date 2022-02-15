use std::{collections::HashMap, vec};

use crate::types::*;

pub fn infer_types(ast: &Vec<Expr>) -> Vec<Expr> {
    let mut inferrer = TypeInferrer::default();
    inferrer.walk_ast(ast)
}

#[derive(Default)]
struct TypeInferrer {
    scoped_variables: HashMap<String, YulType>,
    expected_types: Vec<Option<YulType>>,
}

impl TypeInferrer {
    fn walk_ast(&mut self, ast: &Vec<Expr>) -> Vec<Expr> {
        let mut new_ast = vec![];
        for expr in ast {
            new_ast.push(self.walk_expr(expr.clone()));
        }
        new_ast
    }

    fn walk_expr(&mut self, expr: Expr) -> Expr {
        return match expr {
            //Expr is literal
            Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type,
            })) => Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type: self.expected_types.first().unwrap_or(&None).clone(),
            })),
            Expr::Literal(ref _x) => expr,

            //Expr is function call
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                inferred_return_types,
                inferred_param_types,
                exprs,
            }) => {
                // TODO: this is dumb, but inferring that the params to the function should be the
                // same type as the first return value. Will work for now, for our math and boolean
                // ops
                let expected_param_type = self.expected_types.first().unwrap_or(&None).clone();
                let expressions = exprs
                    .iter()
                    .map(|expr| {
                        self.expected_types = vec![expected_param_type.clone()];
                        self.walk_expr(expr.clone())
                    })
                    .collect();

                Expr::FunctionCall(ExprFunctionCall {
                    function_name,
                    inferred_param_types: exprs
                        .iter()
                        .map(|_| expected_param_type.clone())
                        .collect(),
                    inferred_return_types: self.expected_types.clone(),
                    exprs: Box::new(expressions),
                })
            }

            //Expr is if statement
            Expr::IfStatement(ExprIfStatement {
                first_expr,
                second_expr,
            }) => Expr::IfStatement(ExprIfStatement {
                first_expr: Box::new(self.walk_expr(*first_expr)),
                second_expr: Box::new(ExprBlock {
                    exprs: self.walk_ast(&second_expr.exprs),
                }),
            }),

            //Expr is assignment
            Expr::Assignment(ExprAssignment {
                inferred_types,
                identifiers,
                rhs,
            }) => {
                let inferred_types = identifiers
                    .iter()
                    .map(|ident| Some(self.scoped_variables.get(ident).unwrap().clone()))
                    .collect::<Vec<_>>()
                    .clone();
                self.expected_types = inferred_types.clone();
                Expr::Assignment(ExprAssignment {
                    identifiers,
                    inferred_types,
                    rhs: Box::new(self.walk_expr(*rhs)),
                })
            }

            //Expr is declare variable
            Expr::DeclareVariable(ExprDeclareVariable {
                typed_identifiers,
                rhs,
            }) => {
                // To support shadowing
                for typed_identifier in &typed_identifiers {
                    self.scoped_variables.insert(
                        typed_identifier.identifier.clone(),
                        typed_identifier.yul_type.clone(),
                    );
                }
                self.expected_types = typed_identifiers
                    .iter()
                    .map(|ti| Some(ti.yul_type.clone()))
                    .collect();
                let rhs = rhs.map(|rhs| Box::new(self.walk_expr(*rhs)));
                Expr::DeclareVariable(ExprDeclareVariable {
                    typed_identifiers,
                    rhs,
                })
            }

            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name,
                params,
                returns,
                block,
            }) => {
                let scoped_vars_old = self.scoped_variables.clone();
                for typed_identifier in params.iter().chain(returns.iter()) {
                    self.scoped_variables.insert(
                        typed_identifier.identifier.clone(),
                        typed_identifier.yul_type.clone(),
                    );
                }
                let block = ExprBlock {
                    exprs: self.walk_ast(&block.exprs),
                };
                self.scoped_variables = scoped_vars_old;
                Expr::FunctionDefinition(ExprFunctionDefinition {
                    function_name,
                    params,
                    returns,
                    block,
                })
            }

            Expr::Break => todo!(),

            Expr::Continue => todo!(),
            Expr::Leave => todo!(),

            Expr::Default(ExprDefault { block }) => Expr::Default(ExprDefault {
                block: ExprBlock { exprs: vec![] },
            }),

            //Expr is repeat
            Expr::Repeat(ExprRepeat {
                interior_block,
                iterations,
            }) => Expr::Repeat(ExprRepeat {
                iterations,
                interior_block: Box::new(ExprBlock {
                    exprs: self.walk_ast(&interior_block.exprs),
                }),
            }),

            //Expr is for loop
            Expr::ForLoop(ExprForLoop {
                init_block,
                conditional,
                after_block,
                interior_block,
            }) => {
                let scoped_vars_old = self.scoped_variables.clone();
                let new_expr = Expr::ForLoop(ExprForLoop {
                    init_block: Box::new(ExprBlock {
                        exprs: self.walk_ast(&init_block.exprs),
                    }),
                    conditional: Box::new(self.walk_expr(*conditional)),
                    after_block: Box::new(ExprBlock {
                        exprs: self.walk_ast(&after_block.exprs),
                    }),
                    interior_block: Box::new(ExprBlock {
                        exprs: self.walk_ast(&interior_block.exprs),
                    }),
                });
                self.scoped_variables = scoped_vars_old;
                new_expr
            }

            //Expr is block
            Expr::Block(ExprBlock { exprs }) => {
                let scoped_vars_old = self.scoped_variables.clone();
                let new_expr = Expr::Block(ExprBlock {
                    exprs: self.walk_ast(&exprs),
                });
                self.scoped_variables = scoped_vars_old;
                new_expr
            }

            //Expr is variable
            Expr::Variable(ExprVariableReference {
                identifier,
                inferred_type: _,
            }) => Expr::Variable(ExprVariableReference {
                inferred_type: self.scoped_variables.get(&identifier).cloned(),
                identifier,
            }),
            Expr::Case(_) => todo!(),
        };
    }
}
