mod lib;
extern crate pest;
#[macro_use]
extern crate pest_derive;
use std::collections::HashMap;

// use std::fs;
use pest::iterators::Pair;
use pest::Parser;

#[derive(Parser)]
#[grammar = "./grammar.pest"]
struct IdentParser;

fn main() {
    let yul_code: &str = "gt(add(x,y), add(x,y))";

    let op_codes: Vec<Expr> = transpile(yul_code.to_string());
}

fn transpile(mut syntax: String) -> Vec<Expr> {
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
                Expr::Gt(OpGt {
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

struct Context {
    variables: HashMap<String, u32>,
    next_open_memory_address: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Expr {
    Literal(u128),
    Add(OpAdd),
    Gt(OpGt),
    DeclareVariable(OpDeclareVariable),
    Variable(OpVariable),
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpVariable {
    identifier: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpDeclareVariable {
    identifier: String,
    value: u128,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpAdd {
    first_expr: Box<Expr>,
    second_expr: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OpGt {
    first_expr: Box<Expr>,
    second_expr: Box<Expr>,
}

fn declare_var(program: &mut String, op: &OpDeclareVariable, context: &mut Context) {
    let address = context.next_open_memory_address;
    dbg!(&address);
    context.next_open_memory_address += 1;
    context.variables.insert(op.identifier.clone(), address);
    add_line(program, &format!("push.{}", op.value));
    add_line(program, &format!("mem.store.{}", address));
}

fn add(program: &mut String, op: &OpAdd, context: &mut Context) {
    for expr in [&op.first_expr, &op.second_expr] {
        transpile_op(expr, program, context);
    }
    add_line(program, &format!("add"));
}

fn gt(program: &mut String, op: &OpGt, context: &mut Context) {
    for expr in [&op.first_expr, &op.second_expr] {
        transpile_op(expr, program, context);
    }
    add_line(program, &format!("gt"));
}

fn insert_literal(program: &mut String, value: u128, context: &mut Context) {
    add_line(program, &format!("{}", value));
}

fn load_variable(program: &mut String, op: &OpVariable, context: &mut Context) {
    let address = context.variables.get(&op.identifier).unwrap();
    add_line(program, &format!("mem.load.{}", address));
}

fn add_line(program: &mut String, line: &str) {
    *program = format!("{}\n{}", program, line)
}

fn transpile_op(expr: &Expr, program: &mut String, context: &mut Context) {
    match expr {
        Expr::Literal(value) => insert_literal(program, *value, context),
        Expr::Add(op) => add(program, op, context),
        Expr::Gt(op) => gt(program, op, context),
        Expr::DeclareVariable(op) => declare_var(program, op, context),
        Expr::Variable(op) => load_variable(program, op, context),
    }
}

#[test]
fn test_parse_gt() {
    let yul = "gt(1,2)".to_string();

    let expected_ops = vec![Expr::Gt(OpGt {
        first_expr: Box::new(Expr::Literal(1)),
        second_expr: Box::new(Expr::Literal(2)),
    })];
    assert_eq!(transpile(yul), expected_ops);
}

#[test]
fn test_parse() {
    let yul = "gt(1,2); let x := 12; let y := 15; add(x, y)".to_string();

    let expected_ops = vec![
        Expr::Gt(OpGt {
            first_expr: Box::new(Expr::Literal(1)),
            second_expr: Box::new(Expr::Literal(2)),
        }),
        Expr::DeclareVariable(OpDeclareVariable {
            identifier: "x".to_string(),
            value: 12,
        }),
        Expr::DeclareVariable(OpDeclareVariable {
            identifier: "y".to_string(),
            value: 15,
        }),
        Expr::Add(OpAdd {
            first_expr: Box::new(Expr::Variable(OpVariable {
                identifier: "x".to_string(),
            })),
            second_expr: Box::new(Expr::Variable(OpVariable {
                identifier: "y".to_string(),
            })),
        }),
    ];
    assert_eq!(transpile(yul), expected_ops);
}

#[test]
fn test_add_compilation() {
    let mut program = "begin\npush.0\npush.0\npush.0".to_string();
    let ops = vec![
        Expr::Gt(OpGt {
            first_expr: Box::new(Expr::Literal(1)),
            second_expr: Box::new(Expr::Literal(2)),
        }),
        Expr::DeclareVariable(OpDeclareVariable {
            identifier: "foo".to_string(),
            value: 12,
        }),
        Expr::DeclareVariable(OpDeclareVariable {
            identifier: "bar".to_string(),
            value: 15,
        }),
        Expr::Add(OpAdd {
            first_expr: Box::new(Expr::Variable(OpVariable {
                identifier: "foo".to_string(),
            })),
            second_expr: Box::new(Expr::Variable(OpVariable {
                identifier: "bar".to_string(),
            })),
        }),
    ];
    let mut context = Context {
        variables: HashMap::new(),
        next_open_memory_address: 0,
    };

    for op in ops {
        transpile_op(&op, &mut program, &mut context);
    }
    add_line(&mut program, "end");

    println!("{}", program);
    assert_eq!(
        program,
        "begin
push.0
push.0
push.0
1
2
gt
push.12
mem.store.0
push.15
mem.store.1
mem.load.0
mem.load.1
add
end"
    );
}
