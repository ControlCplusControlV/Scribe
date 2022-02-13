// use std::fs;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::types::*;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

//function to parse yul syntax into miden opcodes
pub fn parse_yul_syntax(syntax: &str) -> Vec<Expr> {
    // Parse the entire file as a string
    let file = IdentParser::parse(Rule::file, &syntax)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    //Parse each statement that matches a grammar pattern inside the file, add them the to Vec<Expr> and return the Vec
    let mut expressions: Vec<Expr> = vec![];
    for statement in file.into_inner() {
        match statement.as_rule() {
            Rule::statement => {
                expressions.push(parse_statement(statement));
            }
            Rule::EOI => (),
            r => {
                dbg!(&statement);
                panic!("Unreachable rule: {:?}", r);
            }
        }
    }
    return expressions;
}

//Function to parse a statement, match a rule defined in grammar.pest and return an Expr
fn parse_statement(expression: Pair<Rule>) -> Expr {
    let inner = expression.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr => parse_expression(inner),
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let identifier = parts.next().unwrap().as_str();
            let rhs = parts.next().unwrap();
            let rhs_expr = parse_expression(rhs);
            return Expr::Assignment(ExprAssignment {
                identifier: identifier.to_string(),
                rhs: Box::new(rhs_expr),
            });
        }
        Rule::variable_declaration => {
            let mut parts = inner.into_inner();
            let identifier_list = parts.next().unwrap();
            let mut identifiers = Vec::new();
            for identifier_rule in identifier_list.into_inner() {
                identifiers.push(identifier_rule.as_str())
            }
            let rhs = parts.next();
            let mut rhs_expr = None;
            if let Some(rhs) = rhs {
                rhs_expr = Some(parse_expression(rhs));
            }
            return Expr::DeclareVariable(ExprDeclareVariable {
                identifier: identifiers.get(0).unwrap().to_string(),
                rhs: rhs_expr.map(Box::new),
            });
        }
        Rule::if_statement => {
            let mut inners = inner.into_inner();
            let first_arg = inners.next().unwrap();
            let second_arg = inners.next().unwrap();
            return Expr::IfStatement(ExprIfStatement {
                first_expr: Box::new(parse_expression(first_arg)),
                second_expr: Box::new(parse_block(second_arg)),
            });
        }
        Rule::for_loop => {
            let mut parts = inner.into_inner();
            let init_block = parts.next().unwrap();
            let conditional = parts.next().unwrap();
            let after_block = parts.next().unwrap();
            let interior_block = parts.next().unwrap();

            return Expr::ForLoop(ExprForLoop {
                init_block: Box::new(parse_block(init_block)),
                conditional: Box::new(parse_expression(conditional)),
                after_block: Box::new(parse_block(after_block)),
                interior_block: Box::new(parse_block(interior_block)),
            });
        }
        r => {
            panic!("Unreachable rule: {:?}", r);
        }
    }
}

fn parse_expression(expression: Pair<Rule>) -> Expr {
    let inner = expression.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::literal => {
            let i = inner.as_str();
            return Expr::Literal(i.parse::<u32>().unwrap());
        }
        Rule::identifier => {
            return Expr::Variable(ExprVariableReference {
                identifier: inner.as_str().to_string(),
            })
        }
        Rule::function_call => {
            let mut inners = inner.into_inner();
            let function_name = get_identifier(inners.next().unwrap());
            let first_arg = inners.next().unwrap();
            let second_arg = inners.next().unwrap();
            return Expr::FunctionCall(ExprFunctionCall {
                function_name: function_name.to_string(),
                first_expr: Box::new(parse_expression(first_arg)),
                second_expr: Box::new(parse_expression(second_arg)),
            });
        }
        r => {
            panic!("Unreachable rule: {:?}", r);
        }
    }
}

fn parse_block(expression: Pair<Rule>) -> ExprBlock {
    let mut exprs: Vec<Expr> = Vec::new();
    for statement in expression.into_inner() {
        exprs.push(parse_statement(statement));
    }

    return ExprBlock { exprs: exprs };
}

fn get_identifier(pair: Pair<Rule>) -> String {
    match pair.as_rule() {
        Rule::identifier => {
            return pair.as_str().to_string();
            // return pair.into_inner().next().unwrap().to_string();
        }
        r => {
            panic!("This was supposed to be an identifier! {:?}", r);
        }
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_var_declaration() {
        let yul = "let x := 1
            let y := 2";

        let expected_ops = vec![
            Expr::DeclareVariable(ExprDeclareVariable {
                identifier: "x".to_string(),
                rhs: Some(Box::new(Expr::Literal(1))),
            }),
            Expr::DeclareVariable(ExprDeclareVariable {
                identifier: "y".to_string(),
                rhs: Some(Box::new(Expr::Literal(2))),
            }),
        ];

        assert_eq!(parse_yul_syntax(yul), expected_ops);
    }

    // #[test]
    // fn parse_var_and_add() {
    //     let yul = "let x := add(1,2)".to_string();

    //     let expected_ops = vec![Expr::DeclareVariable(ExprDeclareVariable {
    //         identifier: "x".to_string(),
    //         rhs: Some(Box::new(Expr::FunctionCall(ExprFunctionCall {
    //             function_name: "add".to_string(),
    //             first_expr: Box::new(Expr::Literal(1)),
    //             second_expr: Box::new(Expr::Literal(2)),
    //         }))),
    //     })];
    //     assert_eq!(parse_yul_syntax(yul), expected_ops);
    // }

    // #[test]
    // fn parse_fibonnaci() {
    //     let yul = "
    // let f := 1
    // let s := 1
    // let next
    // for { let i := 0 } lt(i, 10) { i := add(i, 1)}
    // {
    //   if lt(i, 2) {
    //     mstore(i, 1)
    //   }
    //   if gt(i, 1) {
    //     next := add(s, f)
    //     f := s
    //     s := next
    //     mstore(i, s)
    //   }
    // }"
    //     .to_string();
    //     let res = parse_yul_syntax(yul);
    //     dbg!(&res);
    //     todo!();
    // }
    // #[test]
    // fn parse_if() {
    //     let yul = "
    //   if lt(i, 2) {
    //     mstore(i, 1)
    //   }
    // "
    //     .to_string();
    //     let res = parse_yul_syntax(yul);
    //     dbg!(&res);
    //     todo!();
    // }

    // #[test]
    // fn parse_for_loop() {
    //     let yul = "
    // let f := 1
    // let s := 1
    // let next
    // for { let i := 0 } lt(i, 10) { i := add(i, 1)}
    // {
    //   if lt(i, 2) {
    //     mstore(i, 1)
    //   }
    //   if gt(i, 1) {
    //     next := add(s, f)
    //     f := s
    //     s := next
    //     mstore(i, s)
    //   }
    // }"
    //     .to_string();
    //     let res = parse_yul_syntax(yul);
    //     dbg!(&res);
    //     todo!();
    // }

    //     #[test]
    //     fn parse_cruft() {
    //         let yul = r###"
    // object "fib" {
    //   code {
    //   }
    // }

    //     "###
    //         .to_string();
    //         let res = parse_yul_syntax(yul);
    //         dbg!(&res);
    //         todo!();
    //     }
}
