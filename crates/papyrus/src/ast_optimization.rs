#![allow(dead_code)]
use std::{collections::HashMap, vec};

use crate::types::*;

//TODO: Update this mod and comment the functions

pub fn optimize_ast(ast: Vec<Expr>) -> Vec<Expr> {
    // let mut assignment_visitor = VariableAssignmentVisitor::default();
    // let ast = walk_ast(ast, &mut assignment_visitor);


    // let const_variables = assignment_visitor.get_const_variables();
    // let ast = walk_ast(ast, &mut ConstVariableVisitor { const_variables });


    

    // walk_ast(ast, &mut ForLoopToRepeatVisitor {})
    // TODO: fix optimizations
    ast
}

// Walks through each expression in the abstract syntax tree, optimizing the AST where possible. A new, optimized AST is returned
//Which is then passed into the Miden generation logic.
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

//TODO: Keeps track of constant variables
#[derive(Default)]
struct ConstVariableVisitor {
    const_variables: HashMap<String, ExprLiteral>,
}

#[derive(Default)]
struct ForLoopToRepeatVisitor {}

//The variable assignment visitor keeps track of variables that are reused through the code and the last assignment.
//Variables that do not change can be optimized by converting them into constants.
#[derive(Default)]
struct VariableAssignmentVisitor {
    assignment_counter: HashMap<String, u32>,
    last_assignment: HashMap<String, ExprLiteral>,
}

impl VariableAssignmentVisitor {
    // Checks for variables that are only assigned once and returns a hashmap of the variables to convert into constants.
    fn get_const_variables(&self) -> HashMap<String, ExprLiteral> {
        self.assignment_counter
            .iter()
            .filter(|(_k, v)| **v == 1)
            .filter_map(|(k, _)| {
                if let Some(value) = self.last_assignment.get(k) {
                    return Some((k.clone(), value.clone()));
                }
                None
            })
            .collect::<HashMap<String, ExprLiteral>>()
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
        let preserve_value = Some(expr.clone());

       if let Expr::ForLoop(ExprForLoop {
                init_block,
                conditional,
                after_block,
                interior_block,
            }) = expr.clone() {
                let mut i = 0; // Variable to keep track of where repeat begins
                let mut init_value_variable_name: String = "".to_string();

                //get the init value name, 

                //Get the end value

                // Get the step and see if it is mul, add etc, and get the repeat amount

                // Start out with the init block, and determine where iteration will start
                /// This optimization will only be applied when the init block uses a number literal for the rhs
                if init_block.exprs.len() != 1 {
                    return preserve_value;
                }

               if let Some(Expr::DeclareVariable(ExprDeclareVariable{typed_identifiers, rhs: _}))= init_block.exprs.first(){
                    if let Some(TypedIdentifier{identifier, yul_type}) = typed_identifiers.first(){
                        init_value_variable_name = identifier.to_owned();
                    }
               }else{
                return preserve_value;
               };

                for expr in init_block.exprs {
                    if let Expr::Assignment(value) = expr.clone() {
                        if let Expr::Literal(value) = *value.rhs {
                            match value {
                                ExprLiteral::Number(n)=> {
                                    i = n.value.0[3]; // TODO: make sure this is within u32 constraints
                                }
                                _ => {
                                    return preserve_value;
                                }
                            }
                        }
                    }
                }


                let mut end_val = primitive_types::U256::zero();
                // With `i` now indicating where our iteration will start, we can check the conditional
                if let Expr::FunctionCall(ExprFunctionCall{function_name, exprs, inferred_return_types, inferred_param_types }) = *conditional {

                        if function_name == "lt"{
                            if let Expr::Literal(ExprLiteral::Number(expr_literal_num)) = exprs.last().unwrap() {
                               end_val =  expr_literal_num.value;
                            };
                            
                        } else if function_name == "gt"{
                            if let Expr::Literal(ExprLiteral::Number(expr_literal_num)) = exprs.last().unwrap() {
                                end_val =  expr_literal_num.value;
                             };
                        }else{
                            return preserve_value;
                        }
                    
                }

                // Determine iterator size
                if after_block.exprs.len() != 1 {
                    return preserve_value;
                }
                let mut iterations = 0; // Final value for repeat statement

                for expr in after_block.exprs {
                    match expr {
                        Expr::Assignment(value) => {
                            if let Expr::FunctionCall(ExprFunctionCall { function_name, exprs, .. }) = *value.rhs {
                                match function_name.to_string().as_str() {
                                    "add" => {
                                        let mut step = 1;
                                        for args in *exprs {
                                            match args {
                                                Expr::Literal(ExprLiteral::Number(expr_literal_num)) => {
                                                    step = expr_literal_num.value.0[3]; // TODO: make sure this is within u32 constraints
                                                }
                                                Expr::Variable(ExprVariableReference { identifier , ..}) => {
                                                    if identifier != init_value_variable_name { // Should check identifier is the same as the one which appaered earlier
                                                        return preserve_value;
                                                    }
                                                }
                                                _ => return preserve_value
                                            }
                                        iterations = ((end_val - i) / step).0[3]; // TODO: make sure this is within u32 constraints
                                    }
                                    }
                                    "sub" => {
                                        let mut step = 1;
                                        let args = *exprs;
                                        if let Some(Expr::Variable( ExprVariableReference { identifier , ..})) = args.first() {
                                            if *identifier != init_value_variable_name {
                                                return preserve_value;
                                            }
                                        }

                                        if let Some(Expr::Literal( ExprLiteral::Number())) = args.first() {
                                            if *identifier != init_value_variable_name {
                                                return preserve_value;
                                            }
                                        }
                                        

                                        

                                    }
                                    "mul" => {
                                        let step = 1;
                                        let args = *exprs;            
                                        for args in *exprs {
                                            match args {
                                                Expr::Literal(ExprLiteral::Number(expr_literal_num)) => {
                                                    step = expr_literal_num.value.0[3]; // TODO: make sure this is within u32 constraints
                                                }
                                                Expr::Variable(ExprVariableReference { identifier , ..}) => {
                                                    if identifier != init_value_variable_name { // Should check identifier is the same as the one which appaered earlier
                                                        return preserve_value;
                                                    }
                                                }
                                                _ => return preserve_value
                                            }
                                        }
                                    }
                                    "div" => {
                                        let step = 1;
                                        let args = *exprs;                                        
                                    }
                                    _ => return preserve_value,
                                }
                            }
                        }
                        _ => return Some(expr)
                    }
                }

                //TODO: 

                // for { let i := 29 } lt(i, exponent) { i := add(i, 1) }
                // {
                //     result := mul(result, base)
                // }
                
                return Some(Expr::Repeat(ExprRepeat {
                    iterations: 53,
                    interior_block,
                }));
            };

        Some(expr)
    }
}

impl ExpressionVisitor for VariableAssignmentVisitor {
    fn visit_expr(&mut self, _expr: Expr) -> Option<Expr> {
        todo!();
        // match &expr {
        //     Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs }) => {
        //         if let Some(Expr::Literal(literal)) = rhs.clone().map(|r| *r) {
        //             self.last_assignment.insert(identifier.clone(), literal);
        //         }
        //         let count = self
        //             .assignment_counter
        //             .entry(identifier.clone())
        //             .or_insert(0);
        //         *count += 1;
        //     }
        //     Expr::Assignment(ExprAssignment {
        //         typed_identifier: identifier,
        //         rhs: _,
        //     }) => {
        //         let count = self
        //             .assignment_counter
        //             .entry(identifier.clone())
        //             .or_insert(0);
        //         *count += 1;
        //     }
        //     _ => {}
        // }
        // Some(expr)
    }
}

impl ExpressionVisitor for ConstVariableVisitor {
    fn visit_expr(&mut self, _expr: Expr) -> Option<Expr> {
        todo!();
        // match &expr {
        //     Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs: _ }) => {
        //         if self.const_variables.get(identifier).is_some() {
        //             return None;
        //         }
        //     }
        //     Expr::Variable(ExprVariableReference { identifier }) => {
        //         if let Some(value) = self.const_variables.get(identifier) {
        //             return Some(Expr::Literal(value.clone()));
        //         }
        //     }
        //     _ => {}
        // }
        // Some(expr)
    }
}

// TODO: it would be nice if there wasn't so much cloning in here
fn walk_expr<V: ExpressionVisitor>(expr: Expr, visitor: &mut V) -> Option<Expr> {
    let expr = visitor.visit_expr(expr);
    if let Some(expr) = expr {
        return Some(match expr {
            //Expr is literal
            Expr::Literal(ref _x) => expr,

            //Expr is function call
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                inferred_return_types,
                inferred_param_types,
                exprs,
            }) => Expr::FunctionCall(ExprFunctionCall {
                function_name,
                inferred_return_types,
                inferred_param_types,
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
            Expr::Assignment(ExprAssignment {
                inferred_types,
                identifiers,
                rhs,
            }) => Expr::Assignment(ExprAssignment {
                identifiers,
                inferred_types,
                rhs: Box::new(walk_expr(*rhs, visitor).unwrap()),
            }),

            //Expr is declare variable
            Expr::DeclareVariable(ExprDeclareVariable {
                typed_identifiers,
                rhs,
            }) => Expr::DeclareVariable(ExprDeclareVariable {
                typed_identifiers,
                rhs: rhs.map(|rhs| Box::new(walk_expr(*rhs, visitor).unwrap())),
            }),

            //TODO: Expr is function definition
            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name: _,
                params: _,
                returns: _,
                block: _,
            }) => todo!(),

            //TODO: Expr is break
            Expr::Break => todo!(),

            //TODO: Expr is continue
            Expr::Continue => todo!(),
            Expr::Leave => todo!(),

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
            Expr::Variable(ExprVariableReference {
                identifier: _,
                inferred_type: _,
            }) => expr,
            Expr::Case(_) => todo!(),
            Expr::Switch(_) => todo!(),
        });
    }
    None
}
