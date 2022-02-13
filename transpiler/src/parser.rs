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
    let file = IdentParser::parse(Rule::file, syntax)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    //Parse each statement that matches a grammar pattern inside the file, add them the to Vec<Expr> and return the Vec
    let mut expressions: Vec<Expr> = vec![];
    for statement in file.into_inner() {
        match statement.as_rule() {
            //rule is statement
            Rule::statement => {
                expressions.push(parse_statement(statement));
            }

            //rule is object

            //rule is code

            //rule is data

            //rule is EOI
            Rule::EOI => (),
            r => {
                dbg!(&statement);
                panic!("Unreachable rule: {:?}", r);
            }
        }
    }
    expressions
}

//Function to parse a statement, match a rule defined in grammar.pest and return an Expr
fn parse_statement(expression: Pair<Rule>) -> Expr {
    let inner = expression.into_inner().next().unwrap();
    match inner.as_rule() {
        //rule is expression
        Rule::expr => parse_expression(inner),

        //rule is block
        Rule::block => Expr::Block(parse_block(inner)),

        //rule is function definition
        Rule::function_definition => {
            let mut parts = inner.into_inner();
            let function_name = parts.next().unwrap().to_string();

            //get the typed identifiers from the function and parse each expression
            let typed_identifiers = parts.next().unwrap();
            let mut typed_identifier_list: Vec<Expr> = vec![];
            for identifier in typed_identifiers.into_inner() {
                typed_identifier_list.push(parse_expression(identifier));
            }

            //get the return typed identifiers from the function and parse each expression
            let return_typed_identifiers = parts.next().unwrap();
            let mut return_typed_identifier_list: Vec<Expr> = vec![];
            for identifier in return_typed_identifiers.into_inner() {
                return_typed_identifier_list.push(parse_expression(identifier));
            }

            //get the function block
            let block = parts.next().unwrap();

            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name: function_name,
                typed_identifier_list: typed_identifier_list,
                return_typed_identifier_list: return_typed_identifier_list,
                block: parse_block(block),
            })
        }

        //rule is variable declaration

        //rule is assignment
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let identifier = parts.next().unwrap().as_str();
            let rhs = parts.next().unwrap();
            let rhs_expr = parse_expression(rhs);
            Expr::Assignment(ExprAssignment {
                identifier: identifier.to_string(),
                rhs: Box::new(rhs_expr),
            })
        }

        //rule is if statement
        Rule::if_statement => {
            let mut inners = inner.into_inner();
            let first_arg = inners.next().unwrap();
            let second_arg = inners.next().unwrap();
            Expr::IfStatement(ExprIfStatement {
                first_expr: Box::new(parse_expression(first_arg)),
                second_expr: Box::new(parse_block(second_arg)),
            })
        }

        //rule is switch

        //rule is case

        //rule is default

        //rule is for loop
        Rule::for_loop => {
            let mut parts = inner.into_inner();
            let init_block = parts.next().unwrap();
            let conditional = parts.next().unwrap();
            let after_block = parts.next().unwrap();
            let interior_block = parts.next().unwrap();

            Expr::ForLoop(ExprForLoop {
                init_block: Box::new(parse_block(init_block)),
                conditional: Box::new(parse_expression(conditional)),
                after_block: Box::new(parse_block(after_block)),
                interior_block: Box::new(parse_block(interior_block)),
            })
        }

        //rule is break continue

        //rule is leave

        //rule is variable declaration
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

        //if rule is not defined
        r => {
            panic!("Unreachable rule: {:?}", r);
        }
    }
}

//Function to parse grammar within an expression rule
fn parse_expression(expression: Pair<Rule>) -> Expr {
    let inner = expression.clone().into_inner().next().unwrap();
    match inner.as_rule() {
        //TODO: need to add type name?

        //TODO: need to add type identifier list?

        //TODO: need to add identifier list?

        //if the matched rule is a literal
        Rule::literal => {
            // We're parsing any literal, now we need to recurse because it could be a number,
            // string, true/false, etc.
            parse_expression(inner)
        }
        Rule::number_literal => parse_expression(inner),
        Rule::decimal_number => {
            let i = inner.as_str();
            Expr::Literal(i.parse::<u32>().unwrap())
        }
        Rule::string_literal => {
            todo!()
        }
        Rule::false_literal => {
            todo!()
        }
        Rule::true_literal => {
            todo!()
        }
        //if the matched rule is an identifier
        Rule::identifier => {
            return Expr::Variable(ExprVariableReference {
                identifier: inner.as_str().to_string(),
            })
        }

        //if the matched rule is a function call
        Rule::function_call => {
            let mut inners = inner.into_inner();
            let function_name = get_identifier(inners.next().unwrap());
            let mut exprs: Vec<Expr> = Vec::new();
            // for each argument in the function, parse the expression and add it to exprs
            for arg in inners.into_iter() {
                exprs.push(parse_expression(arg));
            }
            return Expr::FunctionCall(ExprFunctionCall {
                function_name: function_name.to_string(),
                exprs: Box::new(exprs),
            });
        }

        //if the rule has not been defined yet
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

    ExprBlock { exprs }
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

    fn parse_to_tree(yul: &str) -> String {
        expressions_to_tree(&parse_yul_syntax(yul))
    }

    #[test]
    fn parse_var_declaration() {
        insta::assert_snapshot!(parse_to_tree(
            "let x := 1
            let y := 2"
        ));
    }

    #[test]
    fn parse_function_call() {
        insta::assert_snapshot!(parse_to_tree("add(1,2)"));
    }

    #[test]
    fn parse_var_and_add() {
        insta::assert_snapshot!(parse_to_tree("let x := add(1,2)"));
    }

    #[test]
    fn parse_literals() {
        insta::assert_snapshot!(parse_to_tree(
            "
            true
            false
            1
            0x1
            "
        ));
    }

    #[test]
    fn parse_fibonnaci() {
        insta::assert_snapshot!(parse_to_tree(
            "
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
        ));
    }

    #[test]
    fn parse_if() {
        insta::assert_snapshot!(parse_to_tree(
            "
            if lt(i, 2) {
               mstore(i, 1)
            }"
        ));
    }

    #[test]
    fn parse_cruft() {
        let yul = r###"
    object "fib" {
      code {
      }
    }

        "###;
        insta::assert_snapshot!(parse_to_tree(yul));
    }

    //TODO: add test for parse function delcaration
}
