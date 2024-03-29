use std::{collections::HashMap, vec};

use crate::types::*;

//Function to
pub fn infer_types(ast: &Vec<Expr>) -> Vec<Expr> {
    let mut inferrer = TypeInferrer::default();
    inferrer.walk_ast(ast)
}

#[derive(Default)]
struct TypeInferrer {
    scoped_variables: HashMap<String, YulType>,
    expected_types: Vec<Option<YulType>>,
    evaluated_types: Vec<Option<YulType>>,
}

//FIXME: needs comments still
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
            Expr::Literal(literal) => Expr::Literal(self.infer_literal(literal)),

            //Expr is function call
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                inferred_return_types: _,
                inferred_param_types: _,
                exprs,
            }) => {
                // TODO: this is dumb, but inferring that the params to the function should be the
                // same type as the first return value. Will work for now, for our math and boolean
                // ops
                let expected_types = self.expected_types.clone();
                let mut param_types = Vec::new();
                let expressions = exprs
                    .iter()
                    .enumerate()
                    .map(|(i, expr)| {
                        if i == 0 && function_name == "mstore" || function_name == "mload" {
                            self.expected_types = vec![Some(YulType::U32)];
                        }
                        // self.expected_types = vec![expected_param_type.clone()];
                        let new_expr = self.walk_expr(expr.clone());
                        param_types.append(&mut self.evaluated_types.clone());
                        self.expected_types = expected_types.clone();
                        new_expr
                    })
                    .collect();

                let inferred_return_types = self.expected_types.clone();
                self.evaluated_types = inferred_return_types.clone();
                Expr::FunctionCall(ExprFunctionCall {
                    function_name,
                    inferred_param_types: param_types,
                    inferred_return_types,
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
                inferred_types: _,
                identifiers,
                rhs,
            }) => {
                let inferred_types = identifiers
                    .iter()
                    .map(|ident| Some(*self.scoped_variables.get(ident).unwrap()))
                    .collect::<Vec<_>>();
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
                        typed_identifier.yul_type,
                    );
                }
                self.expected_types = typed_identifiers
                    .iter()
                    .map(|ti| Some(ti.yul_type))
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
                        typed_identifier.yul_type,
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
            }) => {
                let inferred_type = self.scoped_variables.get(&identifier).cloned();
                self.evaluated_types = vec![inferred_type];
                Expr::Variable(ExprVariableReference {
                    inferred_type,
                    identifier,
                })
            }
            Expr::Switch(ExprSwitch {
                default_case,
                inferred_type: _,
                expr,
                cases,
            }) => {
                let new_expr = self.walk_expr(*expr);
                let inferred_type = *self.evaluated_types.first().unwrap_or(&Some(YulType::U256));
                let expected_types = self.evaluated_types.clone();
                let cases = cases
                    .into_iter()
                    .map(|case| {
                        self.expected_types = expected_types.clone();
                        ExprCase {
                            literal: self.infer_literal(case.literal),
                            block: ExprBlock {
                                exprs: self.walk_ast(&case.block.exprs),
                            },
                        }
                    })
                    .collect();
                Expr::Switch(ExprSwitch {
                    default_case,
                    inferred_type,
                    expr: Box::new(new_expr),
                    cases,
                })
            }
            _ => unreachable!(),
        };
    }

    fn infer_literal(&mut self, literal: ExprLiteral) -> ExprLiteral {
        match literal {
            ExprLiteral::Number(ExprLiteralNumber {
                value,
                inferred_type: _,
            }) => {
                let inferred_type = *self.expected_types.first().unwrap_or(&Some(YulType::U256));
                self.evaluated_types = vec![inferred_type];
                ExprLiteral::Number(ExprLiteralNumber {
                    value,
                    inferred_type,
                })
            }
            x => x,
        }
    }
}
