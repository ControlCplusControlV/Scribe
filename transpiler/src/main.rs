mod lib;
extern crate pest;
#[macro_use]
extern crate pest_derive;
use std::fs;
use pest::Parser;
use pest::iterators::Pair;


#[derive(Parser)] 
#[grammar = "./grammar.pest"]
struct IdentParser;



fn main() {
    parse_yul_syntax(Rule::greater_than, "gt(200)")   
}



fn parse_yul_syntax(rule: Rule, syntax: &str ) {
    // Parse a string input
    let pair = IdentParser::parse(rule, syntax)
        .expect("unsuccessful parse")
        .next().unwrap();

    print_pair(&pair, true);
    // Iterate over the "inner" Pairs
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::variable_declaration => println!("variable declaration:  {}", inner_pair.as_str()),
            Rule::less_than => println!("lt:  {}", inner_pair.as_str()),
            Rule::greater_than => println!("gt:  {}", inner_pair.as_str()),
            Rule::add => println!("add:  {}", inner_pair.as_str()),
            Rule::mstore => println!("mstore:  {}", inner_pair.as_str()),
            Rule::_if => println!("_if:  {}", inner_pair.as_str()),
           
            _ => unreachable!()
        };    }
}



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


