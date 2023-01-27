use std::collections::HashMap;

use crate::types::{Expr, ExprLiteral};

use super::ExpressionVisitor;

//TODO: Keeps track of constant variables
#[derive(Default)]
struct ConstVariableVisitor {
    const_variables: HashMap<String, ExprLiteral>,
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
