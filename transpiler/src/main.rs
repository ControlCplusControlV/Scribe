mod lib;
extern crate pest;
#[macro_use]
extern crate pest_derive;
// use std::fs;
use pest::Parser;
use pest::iterators::Pair;


#[derive(Parser)] 
#[grammar = "./grammar.pest"]
struct IdentParser;





fn main() {
    let yul_code:&str = "gt(add(x,y), add(x,y))";

    let op_codes: String = transpile(yul_code); 
}


fn transpile(syntax: &str) -> String {
    let mut opcodes: String = "".to_string();


    loop {
        let new_opcode = parse_yul_syntax(syntax);
        opcodes.push_str(new_opcode);

        if syntax.len()==0{
            break;
        }
    }


    return opcodes;


}


fn parse_yul_syntax(syntax: &str) -> &str{
    // Parse a string input
    let pair = IdentParser::parse(Rule::yul_syntax, syntax)
        .expect("unsuccessful parse")
        .next().unwrap();

    print_pair(&pair, true);
    // Iterate over the "inner" Pairs
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            // Rule::variable_declaration => println!("variable declaration:  {}", inner_pair.as_str()),
            // Rule::less_than => println!("lt:  {}", inner_pair.as_str()),
            Rule::greater_than => parse_greater_than(inner_pair.as_span().start(), inner_pair.as_span().end(),  syntax),
            // Rule::add => println!("add:  {}", inner_pair.as_str()),
            // Rule::mstore => println!("mstore:  {}", inner_pair.as_str()),
            // Rule::_if => println!("_if:  {}", inner_pair.as_str()),
            // Rule::inner => println!("inner: {}", inner_pair.as_str()),
            _ => unreachable!()
        };    }

        return "";
}


fn parse_greater_than(start: usize, end: usize, syntax: &str) -> String{
    //return gt opcode and shorten yul syntax by the "stop" position of the rule
 

    return "".to_string();

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


