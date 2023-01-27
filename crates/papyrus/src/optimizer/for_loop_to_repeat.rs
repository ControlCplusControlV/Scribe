//TODO: Update this mod and comment the functions

use crate::types::{
    Expr, ExprDeclareVariable, ExprForLoop, ExprFunctionCall, ExprLiteral, ExprRepeat,
    ExprVariableReference, TypedIdentifier,
};

use super::ExpressionVisitor;

#[derive(Default)]
struct ForLoopToRepeatVisitor {}

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
        }) = expr.clone()
        {
            let mut i = 0; // Variable to keep track of where repeat begins
            let mut init_value_variable_name: String = "".to_string();

            // Get the step and see if it is mul, add etc, and get the repeat amount

            // Start out with the init block, and determine where iteration will start
            /// This optimization will only be applied when the init block uses a number literal for the rhs
            if init_block.exprs.len() != 1 {
                return preserve_value;
            }

            //get the init value name,
            if let Some(Expr::DeclareVariable(ExprDeclareVariable {
                typed_identifiers,
                rhs: _,
            })) = init_block.exprs.first()
            {
                if let Some(TypedIdentifier {
                    identifier,
                    yul_type,
                }) = typed_identifiers.first()
                {
                    init_value_variable_name = identifier.to_owned();
                }
            } else {
                return preserve_value;
            };

            for expr in init_block.exprs {
                if let Expr::Assignment(value) = expr.clone() {
                    if let Expr::Literal(value) = *value.rhs {
                        match value {
                            ExprLiteral::Number(n) => {
                                i = n.value.0[3]; // TODO: make sure this is within u32 constraints
                            }
                            _ => {
                                return preserve_value;
                            }
                        }
                    }
                }
            }

            //Get the end value

            let mut end_val = 0;
            // With `i` now indicating where our iteration will start, we can check the conditional
            if let Expr::FunctionCall(ExprFunctionCall {
                function_name,
                exprs,
                inferred_return_types,
                inferred_param_types,
            }) = *conditional
            {
                if function_name == "lt" {
                    if let Expr::Literal(ExprLiteral::Number(expr_literal_num)) =
                        exprs.last().unwrap()
                    {
                        end_val = expr_literal_num.value.0[3];
                    };
                } else if function_name == "gt" {
                    if let Expr::Literal(ExprLiteral::Number(expr_literal_num)) =
                        exprs.last().unwrap()
                    {
                        end_val = expr_literal_num.value.0[3];
                    };
                } else {
                    return preserve_value;
                }
            }

            // Determine iterator size
            if after_block.exprs.len() != 1 {
                return preserve_value;
            }
            let mut iterations: u32 = 0; // Final value for repeat statement

            for expr in after_block.exprs {
                match expr {
                    Expr::Assignment(value) => {
                        if let Expr::FunctionCall(ExprFunctionCall {
                            function_name,
                            exprs,
                            ..
                        }) = *value.rhs
                        {
                            match function_name.to_string().as_str() {
                                "add" => {
                                    let mut step = 1;
                                    for args in *exprs {
                                        match args {
                                            Expr::Literal(ExprLiteral::Number(
                                                expr_literal_num,
                                            )) => {
                                                step = expr_literal_num.value.0[3];
                                                // TODO: make sure this is within u32 constraints
                                            }
                                            Expr::Variable(ExprVariableReference {
                                                identifier,
                                                ..
                                            }) => {
                                                if identifier != init_value_variable_name {
                                                    // Should check identifier is the same as the one which appaered earlier
                                                    return preserve_value;
                                                }
                                            }
                                            _ => return preserve_value,
                                        }
                                        iterations = ((end_val - i) / step).try_into().unwrap();
                                        // TODO: make sure this is within u32 constraints
                                    }
                                }
                                "sub" => {
                                    let mut step = 1;
                                    let args = *exprs;
                                    if let Some(Expr::Variable(ExprVariableReference {
                                        identifier,
                                        ..
                                    })) = args.first()
                                    {
                                        if *identifier != init_value_variable_name {
                                            return preserve_value;
                                        }
                                    }
                                    if let Some(Expr::Literal(ExprLiteral::Number(num))) =
                                        args.last()
                                    {
                                        step = num.value.0[3];
                                    } else {
                                        return preserve_value;
                                    }

                                    iterations = ((i - end_val) / step).try_into().unwrap();
                                    // Assumes no overflow
                                }
                                "mul" => {
                                    let mut step = 1;
                                    for args in *exprs {
                                        match args {
                                            Expr::Literal(ExprLiteral::Number(
                                                expr_literal_num,
                                            )) => {
                                                step = expr_literal_num.value.0[3];
                                                // TODO: make sure this is within u32 constraints
                                            }
                                            Expr::Variable(ExprVariableReference {
                                                identifier,
                                                ..
                                            }) => {
                                                if identifier != init_value_variable_name {
                                                    // Should check identifier is the same as the one which appaered earlier
                                                    return preserve_value;
                                                }
                                            }
                                            _ => return preserve_value,
                                        }

                                        let mut j = i.clone();
                                        while j < end_val {
                                            j *= step;
                                            iterations += 1;
                                        }
                                    }
                                }
                                "div" => {
                                    let mut step = 1;
                                    for args in *exprs {
                                        match args {
                                            Expr::Literal(ExprLiteral::Number(
                                                expr_literal_num,
                                            )) => {
                                                step = expr_literal_num.value.0[3];
                                                // TODO: make sure this is within u32 constraints
                                            }
                                            Expr::Variable(ExprVariableReference {
                                                identifier,
                                                ..
                                            }) => {
                                                if identifier != init_value_variable_name {
                                                    // Should check identifier is the same as the one which appaered earlier
                                                    return preserve_value;
                                                }
                                            }
                                            _ => return preserve_value,
                                        }

                                        let mut j = i.clone();
                                        while j > end_val {
                                            j /= step;
                                            iterations += 1;
                                        }
                                    }
                                }

                                _ => {}
                            }
                        }
                    }
                    _ => return Some(expr),
                }
            }

            return Some(Expr::Repeat(ExprRepeat {
                iterations,
                interior_block,
            }));
        };

        Some(expr)
    }
}
