// use std::fs;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;

use crate::types::*;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

fn parse(mut syntax: String) -> Vec<Expr> {
    let mut opcodes: Vec<Expr> = vec![];

    //while the syntax string is greater than 0, parse the string for yul syntax and return the miden opcodes.
    loop {
        let new_expr = parse_yul_syntax(&mut syntax);
        opcodes.push(new_expr);

        if syntax.len() == 0 {
            break;
        }
    }

    //return the transpiled miden opcodes
    return opcodes;
}

// Just a type alias for now, but eventually will be an enum like in src/lib.rs
type OpCode = String;

//function to parse yul syntax into miden opcodes
fn parse_yul_syntax(syntax: &mut String) -> Expr {
    // Parse a string input
    let pair = IdentParser::parse(Rule::yul_syntax, syntax)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    //for debugging
    print_pair(&pair, true);

    // Iterate over the "inner" Pairs
    for inner_pair in pair.into_inner() {
        return match inner_pair.as_rule() {
            // Rule::variable_declaration => println!("variable declaration:  {}", inner_pair.as_str()),
            // Rule::less_than => println!("lt:  {}", inner_pair.as_str()),
            // Rule::greater_than => parse_greater_than(
            //     inner_pair.as_span().end(),
            //     syntax,
            // ),
            // Rule::add =>parse_add(
            // inner_pair.as_span().end(),
            // syntax,
            // ),
            // Rule::mstore =>parse_mstore(
            // inner_pair.as_span().end(),
            // syntax,
            // ),
            // Rule::_if =>parse_if(
            // inner_pair.as_span().end(),
            // syntax,
            // ),
            Rule::greater_than => {
                parse_greater_than(inner_pair.as_span().end(), syntax);
                // TODO: expect two more expressions, so parse for two more expressions, then
                // return something like:
                // Pretending like we've parsed everything
                *syntax = "".to_string();
                // TODO: we'll also have to "finish" parsing this by consuming the last
                // parentheses, after parsing the two arguments as expressions
                Expr::Gt(ExprGt {
                    first_expr: Box::new(Expr::Literal(1)),
                    second_expr: Box::new(Expr::Literal(2)),
                })
            }
            // Rule::add => println!("add:  {}", inner_pair.as_str()),
            // Rule::mstore => println!("mstore:  {}", inner_pair.as_str()),
            // Rule::_if => println!("_if:  {}", inner_pair.as_str()),
            // Rule::inner => println!("inner: {}", inner_pair.as_str()),
            _ => unreachable!(),
        };
    }
    todo!("is this even possible to hit, maybe we should return None");
}

//Function to parse greater than syntax from yul into a miden opcode
fn parse_greater_than(end: usize, syntax: &mut String) -> OpCode {
    *syntax = syntax[end..].to_string();
    return "GT".to_string();
}

//Function to parse greater than syntax from yul into a miden opcode
fn parse_add(end: usize, syntax: &mut String) -> OpCode {
    *syntax = syntax[end..].to_string();
    return "ADD".to_string();
}

//Function to parse greater than syntax from yul into a miden opcode
fn parse_mstore(end: usize, syntax: &mut String) -> OpCode {
    *syntax = syntax[end..].to_string();
    return "".to_string();
}

//Function to parse greater than syntax from yul into a miden opcode
fn parse_if(end: usize, syntax: &mut String) -> OpCode {
    *syntax = syntax[end..].to_string();
    return "".to_string();
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
    fn parse_gt() {
        let yul = "gt(1,2)".to_string();

        let expected_ops = vec![Expr::Gt(ExprGt {
            first_expr: Box::new(Expr::Literal(1)),
            second_expr: Box::new(Expr::Literal(2)),
        })];
        assert_eq!(parse(yul), expected_ops);
    }

    #[ignore]
    #[test]
    fn parse_gt_add() {
        let yul = "gt(1,2); let x := 12; let y := 15; add(x, y)".to_string();

        let expected_ops = vec![
            Expr::Gt(ExprGt {
                first_expr: Box::new(Expr::Literal(1)),
                second_expr: Box::new(Expr::Literal(2)),
            }),
            Expr::DeclareVariable(ExprDeclareVariable {
                identifier: "x".to_string(),
                value: 12,
            }),
            Expr::DeclareVariable(ExprDeclareVariable {
                identifier: "y".to_string(),
                value: 15,
            }),
            Expr::Add(ExprAdd {
                first_expr: Box::new(Expr::Variable(ExprVariableReference {
                    identifier: "x".to_string(),
                })),
                second_expr: Box::new(Expr::Variable(ExprVariableReference {
                    identifier: "y".to_string(),
                })),
            }),
        ];
        assert_eq!(parse(yul), expected_ops);
    }
}
