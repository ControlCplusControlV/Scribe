// use std::fs;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::types::*;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

//function to parse yul syntax into miden opcodes
pub fn parse_yul_syntax(syntax: &mut String) -> Vec<Expr> {
    // Parse a string input
    let file = IdentParser::parse(Rule::file, syntax)
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
        Rule::variable_declaration => {
            let mut parts = inner.into_inner();
            let identifier = parts.next().unwrap().as_str();
            let rhs = parse_expression(parts.next().unwrap());
            return Expr::DeclareVariable(ExprDeclareVariable {
                identifier: identifier.to_string(),
                rhs: Box::new(rhs),
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
        Rule::function_call => {
            let mut inners = inner.into_inner();
            let function_name = get_identifier(inners.next().unwrap());
            let first_arg = inners.next().unwrap();
            let second_arg = inners.next().unwrap();
            dbg!(&function_name);
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

//for debugging
fn print_pair(pair: &Pair<Rule>, hard_divider: bool) {
    println!("Rule: {:?}", pair.as_rule());
    println!("Span: {:?}", pair.as_span());
    println!("Text: {:?}", pair.as_str());
    if hard_divider {
        println!("{:=>60}", "");
    } else {
        println!("{:->60}", "");
    }
}

// TESTS
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_var_declaration() {
        let mut yul = "let x := 1".to_string();

        let expected_ops = vec![Expr::DeclareVariable(ExprDeclareVariable {
            identifier: "x".to_string(),
            rhs: Box::new(Expr::Literal(1)),
        })];
        assert_eq!(parse_yul_syntax(&mut yul), expected_ops);
    }

    #[test]
    fn parse_var_and_add() {
        let mut yul = "let x := add(1,2)".to_string();

        let expected_ops = vec![Expr::DeclareVariable(ExprDeclareVariable {
            identifier: "x".to_string(),
            rhs: Box::new(Expr::FunctionCall(ExprFunctionCall {
                function_name: "add".to_string(),
                first_expr: Box::new(Expr::Literal(1)),
                second_expr: Box::new(Expr::Literal(2)),
            })),
        })];
        assert_eq!(parse_yul_syntax(&mut yul), expected_ops);
    }
}
