use core::num::dec2flt::parse;

// use std::fs;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::types::*;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

//function to parse yul syntax into miden opcodes
pub fn parse_yul_syntax(syntax: String) -> Vec<Expr> {
    // Parse a string input
    let file = IdentParser::parse(Rule::file, &syntax)
        .expect("unsuccessful parse")
        .next()
        .unwrap();
    let mut expressions: Vec<Expr> = vec![];
    for statement in file.into_inner() {
        match statement.as_rule() {
            Rule::statement => {
                expressions.push(parse_statement(statement));
            }
            Rule::EOI => (),
            r => {
                panic!("Unreachable rule: {:?}", r);
            }
        }
    }
    return expressions;
}

fn parse_statement(expression: Pair<Rule>) -> Expr {
    let inner = expression.into_inner().next().unwrap();
    match inner.as_rule() {
        Rule::expr => parse_expression(inner),
        Rule::variable_declaration => {
            let mut parts = inner.into_inner();
            let identifier = parts.next().unwrap().as_str();
            let rhs = parts.next();
            let mut rhs_expr = None;
            if let Some(rhs) = rhs {
                rhs_expr = Some(parse_expression(rhs));
            }
            return Expr::DeclareVariable(ExprDeclareVariable {
                identifier: identifier.to_string(),
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

            return Expr::ForLoop(ExprForLoop{
                 init_block: Box::new(parse_block(init_block)),
                 conditional:  Box::new(parse_expression(conditional)),
                 after_block: Box::new(parse_block(after_block)),
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
        Rule::literal => return Expr::Literal(inner.as_str().parse::<u128>().unwrap()),
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
        let mut yul = "let x := 1
            let y := 2"
            .to_string();

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

    #[test]
    fn parse_var_and_add() {
        let mut yul = "let x := add(1,2)".to_string();

        let expected_ops = vec![Expr::DeclareVariable(ExprDeclareVariable {
            identifier: "x".to_string(),
            rhs: Some(Box::new(Expr::FunctionCall(ExprFunctionCall {
                function_name: "add".to_string(),
                first_expr: Box::new(Expr::Literal(1)),
                second_expr: Box::new(Expr::Literal(2)),
            }))),
        })];
        assert_eq!(parse_yul_syntax(yul), expected_ops);
    }

    #[test]
    fn parse_fibonnaci() {
        let mut yul = "
    let f := 1
    let s := 1
    let next
    for { let i := 0 } lt(i, 10) { i := add(i, 1)}
    {
      if lt(i, 2) {
        mstore(i, 1)
      }
      if gt(i, 1) {
        next := add(s, f)
        f := s
        s := next
        mstore(i, s)
      }
    }"
        .to_string();
        let res = parse_yul_syntax(yul);
        dbg!(&res);
        todo!();
    }
    #[test]
    fn parse_if() {
        let mut yul = "
      if lt(i, 2) {
        mstore(i, 1)
      }
    "
        .to_string();
        let res = parse_yul_syntax(yul);
        dbg!(&res);
        todo!();
    }


    #[test]
    fn parse_for_loop() {
        let mut yul = "
        for { let i := 0 } lt(i, 10) { i := add(i, 1)}"
        .to_string();
        let res = parse_yul_syntax(yul);
        dbg!(&res);
        todo!();
    }
}
