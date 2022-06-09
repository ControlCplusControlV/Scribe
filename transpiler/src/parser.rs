use crate::types::*;
use pest::iterators::Pair;
use pest::Parser;
use pest_derive::Parser;
use primitive_types::U256;
use std::str;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;
const DEFAULT_TYPE: YulType = YulType::U256;

//Takes in yul code as a string and parses the grammar, returning a Struct that represents a statement or expression in Yul
//Yul grammar is parsed by matching rules, which can be found in the grammar.pest file
//After a rule is matched, the statement or expression is unwrapped to parse nested rules.
//For example, a the grammar for a decimal_number is @{ digit+ }, and a digit is { '0'..'9' }

//To see examples for each Expr, check out types.rs
pub fn parse_yul_syntax(syntax: &str) -> Vec<Expr> {
    let file = IdentParser::parse(Rule::file, syntax)
        .expect("unsuccessful parse")
        .next()
        .unwrap();

    //Parse each statement that matches a grammar pattern inside the file, add them the to Vec<Expr> and return the Vec
    let mut expressions: Vec<Expr> = vec![];
    for statement in file.clone().into_inner() {
        match statement.as_rule() {
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

            Rule::EOI => (),
            r => {
                dbg!(&statement);
                panic!("Unreachable rule: {:?}", r);
            }
        }
    }
    expressions
}

//Parses a Yul statement. This function matches a grammar rule and return an Expr struct
//which is later added into the Abstract Syntax Tree
fn parse_statement(expression: Pair<Rule>) -> Expr {
    let inner = expression.into_inner().next().unwrap();
    match inner.as_rule() {
        //Rule is expr
        Rule::expr => parse_expression(inner),

        //Rule is block
        Rule::block => Expr::Block(parse_block(inner)),

        // Rule is code
        Rule::code => Expr::Block(parse_block(inner.into_inner().next().unwrap())),

        //If the rule is a function definition, parse the function name, parameters, returns and then return an Expr
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

        //Rule is assignment
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

        //Rule is if statement
        Rule::if_statement => {
            let mut inners = inner.into_inner();
            let first_arg = inners.next().unwrap();
            let second_arg = inners.next().unwrap();
            Expr::IfStatement(ExprIfStatement {
                first_expr: Box::new(parse_expression(first_arg)),
                second_expr: Box::new(parse_block(second_arg)),
            })
        }

        //Rule is switch
        Rule::switch => {
            let mut parts = inner.into_inner();
            let mut default_case = None;
            let mut cases = Vec::new();
            let expr = parse_expression(parts.next().unwrap());
            for part in parts {
                match part.as_rule() {
                    Rule::case => cases.push(parse_case(part)),
                    Rule::default => {
                        default_case = Some(parse_block(part.into_inner().next().unwrap()))
                    }
                    _ => unreachable!(),
                }
            }

            Expr::Switch(ExprSwitch {
                expr: Box::new(expr),
                inferred_type: None,
                cases,
                default_case,
            })
        }

        //Rule is case
        Rule::case => {
            let mut parts = inner.into_inner();
            let literal = parts.next().unwrap();
            let block = parts.next().unwrap();

            Expr::Case(ExprCase {
                literal: parse_literal(literal),
                block: parse_block(block),
            })
        }

        //Rule is for loop
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

        //Rule is break
        Rule::break_ => Expr::Break,

        //Rule is continue
        Rule::continue_ => Expr::Continue,

        //Rule is leave
        Rule::leave => Expr::Leave,

        //Rule is variable declaration
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

//Parses an identifier list for function definitions or variable declarations.
//TODO: explain how this gets handled in transpilation, variables stored in a hashmap during translation
fn parse_identifier_list(rule: Pair<Rule>) -> Vec<Identifier> {
    let mut identifiers = Vec::new();
    for rule in rule.into_inner() {
        let identifier = rule.as_str();
        identifiers.push(identifier.to_string());
    }
    identifiers
}

//Parses a case statement into an Expr
fn parse_case(rule: Pair<Rule>) -> ExprCase {
    let mut parts = rule.into_inner();
    let literal = parse_literal(parts.next().unwrap());
    let block = parse_block(parts.next().unwrap());
    ExprCase { block, literal }
}

//Parses a typed identifier list for function definitions or variable declarations. This is later used to determine
//what type of operation to use for specific instructions (ex. u256add vs u32add).
//Currently the two Yul types that are supported are u32 and u256
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
    }
    identifiers
}

//Parses a literal into an Expr
//Literals can be a number literal, string literal, true/false literal or a hex literal.
//Literals can also have an optional type in Yul.
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
        Rule::literal => {
            // Parsing literals need to recurse because it could be a number literal
            Expr::Literal(parse_literal(expression))
        }
        Rule::number_literal => parse_expression(expression),
        Rule::hex_number => {
            // TODO: parse hex numbers
            let mut initial = expression.as_str();
            Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: None,
                value: U256::from_str_radix(initial, 16).unwrap(),
            }))
        }
        Rule::hex_literal => {
            // TODO: parse hex numbers
            let initial = expression.as_str();
            Expr::Literal(ExprLiteral::Number(ExprLiteralNumber {
                inferred_type: None,
                value: U256::from_str_radix(initial, 16).unwrap(),
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

//Parses an identifier into an Expr, which gets transpiled into a variable reference.
//These variables need to be kept track of during transpilation in case their value changes during runtime,
// which needs to be accounted for during transpilation.
fn parse_identifier(identifier: Pair<Rule>) -> Expr {
    return Expr::Variable(ExprVariableReference {
        identifier: identifier.as_str().to_string(),
        inferred_type: None,
    });
}

//Parses a block into an Expr
fn parse_block(expression: Pair<Rule>) -> ExprBlock {
    let mut exprs: Vec<Expr> = Vec::new();
    for statement in expression.into_inner() {
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

    #[test]
    fn parse_switch_statement() {
        insta::assert_snapshot!(parse_to_tree(
            "
            let x := 5
            let y := 8
            switch x
                case 3 {
                    y := 5
                }
                case 5 {
                    y := 12
                    let z := 15
                }
                case 8 {
                    y := 15
                }
            y"
        ));
    }

    //TODO: add test for default
}
