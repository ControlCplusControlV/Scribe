// use std::fs;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use primitive_types::U256;

use crate::type_inference::infer_types;
use crate::types::*;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

const DEFAULT_TYPE: YulType = YulType::U32;

//function to parse yul syntax into miden opcodes
pub fn parse_yul_syntax(syntax: &str) -> Vec<Expr> {
    // Parse the entire file as a string
    let file = IdentParser::parse(Rule::file, syntax)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    //Parse each statement that matches a grammar pattern inside the file, add them the to Vec<Expr> and return the Vec
    let mut expressions: Vec<Expr> = vec![];
    for statement in file.clone().into_inner() {
        match statement.as_rule() {
            //rule is statement
            Rule::statement => {
                expressions.push(parse_statement(statement));
            }
            Rule::object => {
                // TODO: create an object type
                let mut parts = statement.into_inner();
                let object_name = parts.next().unwrap();
                dbg!(&object_name);
                let code = parts.next().unwrap();
                expressions.push(parse_statement(code));
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
        Rule::code => Expr::Block(parse_block(inner.into_inner().next().unwrap())),

        //rule is function definition
        Rule::function_definition => {
            let mut parts = inner.into_inner();
            let function_name = parts.next().unwrap().as_str();

            //get the typed identifiers from the function and parse each expression
            let params: Vec<TypedIdentifier> = parse_typed_identifier_list(parts.next().unwrap());
            let returns_rule = parts.next().unwrap();
            let mut returns = vec![];
            if let Some(inner) = returns_rule.into_inner().next() {
                returns = parse_typed_identifier_list(inner);
            }

            let block = parts.next().unwrap();

            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name: function_name.to_string(),
                params,
                returns,
                block: parse_block(block),
            })
        }

        //rule is assignment
        Rule::assignment => {
            let mut parts = inner.into_inner();
            let identifiers = parse_identifier_list(parts.next().unwrap());
            let rhs = parts.next().unwrap();
            let rhs_expr = parse_expression(rhs);
            Expr::Assignment(ExprAssignment {
                identifiers,
                inferred_types: vec![],
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

        // // rule is case
        Rule::case => {
            let mut parts = inner.into_inner();
            let literal = parts.next().unwrap();
            let block = parts.next().unwrap();

            Expr::Case(ExprCase {
                literal: parse_literal(literal),
                block: parse_block(block),
            })
        }

        //rule is default
        Rule::default => {
            let mut parts = inner.into_inner();
            let block = parts.next().unwrap();
            Expr::Default(ExprDefault {
                block: parse_block(block),
            })
        }

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

        //rule is break
        Rule::break_ => Expr::Break,

        //rule is leave
        Rule::continue_ => Expr::Continue,
        Rule::leave => Expr::Leave,

        //rule is variable declaration
        Rule::variable_declaration => {
            let mut parts = inner.into_inner();
            let typed_identifiers: Vec<TypedIdentifier> =
                parse_typed_identifier_list(parts.next().unwrap());
            let rhs = parts.next();
            let mut rhs_expr = None;
            if let Some(rhs) = rhs {
                rhs_expr = Some(parse_expression(rhs));
            }
            return Expr::DeclareVariable(ExprDeclareVariable {
                typed_identifiers: typed_identifiers,
                rhs: rhs_expr.map(Box::new),
            });
        }

        //if rule is not defined
        r => {
            panic!("Unreachable rule: {:?}", r);
        }
    }
}

fn parse_identifier_list(rule: Pair<Rule>) -> Vec<Identifier> {
    let mut identifiers = Vec::new();
    for rule in rule.into_inner() {
        let identifier = rule.as_str();
        identifiers.push(identifier.to_string());
    }
    identifiers
}

fn parse_typed_identifier_list(rule: Pair<Rule>) -> Vec<TypedIdentifier> {
    let mut identifiers = Vec::new();
    for rules in rule.into_inner() {
        let mut parts = rules.into_inner();
        let identifier = parts.next().unwrap().as_str();
        let yul_type = parts
            .next()
            .map(|x| YulType::from_annotation(x.as_str()))
            .unwrap_or(DEFAULT_TYPE);
        identifiers.push(TypedIdentifier {
            identifier: identifier.to_string(),
            yul_type,
        })
        // identifiers.push(part.as_str().to_string());
    }
    identifiers
}

fn parse_literal(literal: Pair<Rule>) -> ExprLiteral {
    match parse_expression(literal.clone()) {
        Expr::Literal(literal) => literal,
        _ => unreachable!("This should only parse literals {:?}", &literal),
    }
}

//Function to parse grammar within an expression rule
fn parse_expression(expression: Pair<Rule>) -> Expr {
    let expression = expression.clone().into_inner().next().unwrap();
    match expression.as_rule() {
        // Rule::expr => parse_expression(expression.into_inner().next().unwrap()),
        //TODO: need to add type name?

        //TODO: need to add type identifier list?

        //TODO: need to add identifier list?

        //if the matched rule is a literal
        Rule::literal => {
            // We're parsing any literal, now we need to recurse because it could be a number,
            // string, true/false, etc.
            Expr::Literal(parse_literal(expression))
        }
        Rule::number_literal => parse_expression(expression),
        Rule::hex_number => {
            // TODO: parse hex numbers
            let i = expression.as_str();
            Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: None,
                value: U256::MAX,
            }))
        }
        Rule::hex_literal => {
            // TODO: parse hex numbers
            let i = expression.as_str();
            Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: None,
                value: U256::MAX,
            }))
        }
        Rule::decimal_number => {
            let i = expression.as_str();
            Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: None,
                value: U256::from_dec_str(i).unwrap(),
            }))
        }
        Rule::string_literal => {
            let content = expression.into_inner().next().unwrap();
            Expr::Literal(ExprLiteral::String(content.as_str().to_string()))
        }

        // //rule is a false literal
        // Rule::false_literal => Expr::Bool(false),

        // //rule is a true literal
        // Rule::true_literal => Expr::Bool(true),

        //if the matched rule is an identifier
        Rule::identifier => parse_identifier(expression),

        //if the matched rule is a function call
        Rule::function_call => {
            let mut inners = expression.into_inner();
            let function_name = inners.next().unwrap().as_str();
            let mut exprs: Vec<Expr> = Vec::new();
            // for each argument in the function, parse the expression and add it to exprs
            for arg in inners.into_iter() {
                exprs.push(parse_expression(arg));
            }
            return Expr::FunctionCall(ExprFunctionCall {
                function_name: function_name.to_string(),
                exprs: Box::new(exprs),
                inferred_return_types: vec![],
                inferred_param_types: vec![],
            });
        }

        //if the rule has not been defined yet
        r => {
            panic!("Unreachable rule: {:?}", r);
        }
    }
}

fn parse_identifier(identifier: Pair<Rule>) -> Expr {
    return Expr::Variable(ExprVariableReference {
        identifier: identifier.as_str().to_string(),
        inferred_type: None,
    });
}

fn parse_block(expression: Pair<Rule>) -> ExprBlock {
    let mut exprs: Vec<Expr> = Vec::new();
    for statement in expression.into_inner() {
        // for comments, probably better solution here
        if statement.clone().into_inner().next().is_some() {
            exprs.push(parse_statement(statement));
        }
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
        let ast = parse_yul_syntax(yul);
        let ast_with_inferred_types = infer_types(&ast);
        expressions_to_tree(&ast_with_inferred_types)
    }

    #[test]
    fn parse_var_declaration() {
        insta::assert_snapshot!(parse_to_tree(
            "let x := 1
            let y := 2"
        ));
    }

    #[test]
    fn parse_var_declaration_with_types() {
        insta::assert_snapshot!(parse_to_tree(
            "let x:u32 := 1
            let y:u256 := 2
            let z := 2
            "
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

    #[ignore]
    #[test]
    fn parse_literals() {
        insta::assert_snapshot!(parse_to_tree(
            "
            \"string_literal\"
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
            }
            "
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

    //TODO: add test for parse function definition

    #[ignore]
    #[test]
    fn parse_break() {
        insta::assert_snapshot!(parse_to_tree(
            "for { let i := 0 } lt(i, 10) { i := add(i, 1)}
        {
            if lt(i,3){
                break
            }
        "
        ));
    }

    #[ignore]
    #[test]
    fn parse_continue() {
        insta::assert_snapshot!(parse_to_tree(
            "for { let i := 0 } lt(i, 10) { i := add(i, 1)}
        {
            if lt(i,3){
                continue
            }
        "
        ));
    }

    #[test]
    fn parse_function_def_with_return() {
        insta::assert_snapshot!(parse_to_tree(
            "
            function allocate_unbounded() -> memPtr {
                memPtr := mload(64)
            }"
        ));
    }

    #[test]
    fn parse_function_def_without_return() {
        insta::assert_snapshot!(parse_to_tree(
            "
            function allocate_unbounded()  {
                let memPtr := mload(64)
            }"
        ));
    }

    //TODO: add test for default
}
