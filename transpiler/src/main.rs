mod lib;
extern crate pest;
#[macro_use]
extern crate pest_derive;
// use std::fs;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

fn main() {
    let yul_code: &str = "gt(add(x,y), add(x,y))";

    let op_codes: Vec<OpCode> = transpile(yul_code.to_string());
}

fn transpile(mut syntax: String) -> Vec<OpCode> {
    let mut opcodes: Vec<OpCode> = vec![];

    //while the syntax string is greater than 0, parse the string for yul syntax and return the miden opcodes.
    loop {
        let new_opcode = parse_yul_syntax(&mut syntax);
        opcodes.push(new_opcode);

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
fn parse_yul_syntax(syntax: &mut String) -> OpCode {
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
            Rule::greater_than => parse_greater_than(
                inner_pair.as_span().start(),
                inner_pair.as_span().end(),
                syntax,
            ),
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
fn parse_greater_than(start: usize, end: usize, syntax: &mut String) -> OpCode {
    *syntax = syntax[end..].to_string();
    return "GT".to_string();
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
