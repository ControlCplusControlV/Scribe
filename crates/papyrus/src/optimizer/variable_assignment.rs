use std::collections::HashMap;

use crate::types::{Expr, ExprLiteral};

use super::ExpressionVisitor;

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
