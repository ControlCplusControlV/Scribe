use crate::types::{
    Expr, ExprAssignment, ExprBlock, ExprDeclareVariable, ExprForLoop, ExprFunctionCall,
    ExprFunctionDefinition, ExprIfStatement, ExprRepeat, ExprVariableReference,
};

pub mod const_variable;
pub mod for_loop_to_repeat;
pub mod variable_assignment;

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

trait ExpressionVisitor {
    fn visit_expr(&mut self, expr: Expr) -> Option<Expr>;
}
