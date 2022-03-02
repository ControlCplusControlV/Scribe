use std::fmt;

use primitive_types::U256;

//Enum to represent Yul Expressions
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Literal(ExprLiteral),
    FunctionDefinition(ExprFunctionDefinition),
    FunctionCall(ExprFunctionCall),
    IfStatement(ExprIfStatement),
    Assignment(ExprAssignment),
    DeclareVariable(ExprDeclareVariable),
    ForLoop(ExprForLoop),
    Block(ExprBlock),
    Switch(ExprSwitch),
    Case(ExprCase),
    Variable(ExprVariableReference),
    Repeat(ExprRepeat),
    Break,
    Continue,
    Leave,
}

impl Expr {
    pub fn get_inferred_type(&self) -> Option<YulType> {
        match self {
            Expr::Literal(ExprLiteral::Number(x)) => x.inferred_type.clone(),
            Expr::Literal(_) => todo!(),
            Expr::FunctionCall(x) => x.inferred_return_types.first().unwrap().clone(),
            Expr::Variable(x) => x.inferred_type.clone(),
            _ => unreachable!(),
        }
    }
}

//Type to represent u32 and u256 integers
#[derive(Hash, Clone, PartialEq, Eq, Debug, Copy)]
pub enum YulType {
    U32,
    U256,
}

impl YulType {
    //Converts a string representation of u32 or u256 to a YulType
    pub fn from_annotation(annotation: &str) -> Self {
        dbg!(annotation);
        match annotation {
            "u32" => Self::U32,
            "u256" => Self::U256,
            _ => panic!(),
        }
    }

    //Returns the stack width that the uint occupies in the Miden VM
    //Miden stack elements can occupy 32bits, a u32 number will occupy one element, where a u256 number will occupy 8 elements
    //u256 numbers are stored in little endian
    pub fn miden_stack_width(&self) -> u32 {
        match self {
            Self::U32 => 1,
            Self::U256 => 8,
        }
    }

    //Returns the amount of addresses in memory the number occupies.
    //Memory addresses in Miden are four words (one word is 32bits).
    //A u32 number will take up one word, meaning that it will only need one address.
    //A u256 number will take up 8 words, meaning that it will need two addresses.
    pub fn miden_memory_addresses(&self) -> u32 {
        match self {
            Self::U32 => 1,
            Self::U256 => 2,
        }
    }
}

//Enum to represent Yul literals
//Number Literal Ex: 123456789 (this gets converted into an ExprLiteralNumber)
//String Literal Ex: "hello world"
//True/False Literal Ex: True
//TODO: Hex literals
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprLiteral {
    Number(ExprLiteralNumber),
    String(String),
    Bool(bool),
}

//Struct to represent a number literal
//If the number has a type, it is stored as a YulType in the inferred_type param.
//Ex: let:u256 = 123456789
//The value is always stored as U256 during parsing because the default data type is u256. During Miden generation if the inferred_type
//is u32, then it is pushed to the stack as a u32 instead.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprLiteralNumber {
    pub inferred_type: Option<YulType>,
    pub value: U256,
}

//Struct to represent a variable reference
//Ex. let x := 1234
//Can also accept inferred types for u32 and u256
// let zero:u32 := 0:u32
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprVariableReference {
    pub identifier: String,
    pub inferred_type: Option<YulType>,
}

//Struct to represent a switch expression
//Ex.
// switch x
// case 0 { result := 1 }
// case 1 { result := 2 }
// default {
//     result := add(1,2)
// }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprSwitch {
    pub default_case: Option<ExprBlock>,
    pub inferred_type: Option<YulType>,
    pub expr: Box<Expr>,
    pub cases: Vec<ExprCase>,
}

//Struct to represent a case block during a switch statement
//Ex. case 0 { result := 1 }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprCase {
    pub literal: ExprLiteral,
    pub block: ExprBlock,
}

//Struct to represent a function definition
//Ex. function f(a,b) -> c, d { }
//Params and return variables can also be typed
//Ex. function f(a:u256, b:u256) -> c:u256, d:u256 { }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprFunctionDefinition {
    pub function_name: String,
    pub params: Vec<TypedIdentifier>,
    pub returns: Vec<TypedIdentifier>,
    pub block: ExprBlock,
}

//Struct to represent a block, consisting of a Vec of expressions
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBlock {
    pub exprs: Vec<Expr>,
}

//Struct to represent variable assignment
//Ex. x = 1234
//Variable types can also be defined
//Ex. x = 1234:u256
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprAssignment {
    pub identifiers: Vec<String>,
    pub inferred_types: Vec<Option<YulType>>,
    pub rhs: Box<Expr>,
}

//Struct to represent break/continue statement
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBreakContinue {
    pub identifier: String,
    pub rhs: Box<Expr>,
}

//Struct to represent a for loop
//Ex.
//{
//     let x := 0
//     for { let i := 0 } lt(i, 0x100) { i := add(i, 0x20) } {
//         x := add(x, mload(i))
//     }
// }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprForLoop {
    pub init_block: Box<ExprBlock>,
    pub conditional: Box<Expr>,
    pub after_block: Box<ExprBlock>,
    pub interior_block: Box<ExprBlock>,
}

//
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprRepeat {
    pub interior_block: Box<ExprBlock>,
    pub iterations: u32,
}

//Struct to represent variable declaration
//Ex. let x := 1234
//Variable types can also be defined
//Ex. x:u256 := 1234
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprDeclareVariable {
    pub typed_identifiers: Vec<TypedIdentifier>,
    pub rhs: Option<Box<Expr>>,
}

//Struct to represent if statement
//Ex.
//if lt(a, b) { sstore(0, 1) }
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprIfStatement {
    pub first_expr: Box<Expr>,
    pub second_expr: Box<ExprBlock>,
}

//Struct to represent a function call
//Ex.
//let c := add(a, b)
//Params and return types can also be defined
//let c:u256 := add(a:u256, b:u256)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprFunctionCall {
    pub function_name: String,
    pub exprs: Box<Vec<Expr>>,
    pub inferred_return_types: Vec<Option<YulType>>,
    pub inferred_param_types: Vec<Option<YulType>>,
}

//Struct to represent a typed identifier
//Ex. x:u256
#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct TypedIdentifier {
    pub identifier: String,
    pub yul_type: YulType,
}

pub type Identifier = String;

use debug_tree::{add_branch_to, add_leaf_to, TreeBuilder, TreeSymbols};

//Implementations for the Expr enum
impl Expr {
    //Add the Expr to the abstract syntax tree. Each Expr is added as a tree "leaf".
    fn add_to_tree(&self, tree: &mut TreeBuilder) {
        match self {
            //--------------------------------------------------------
            //is literal
            Expr::Literal(literal) => match literal {
                ExprLiteral::Number(ExprLiteralNumber {
                    inferred_type,
                    value,
                }) => tree.add_leaf(&format!(
                    "{}:{}",
                    value,
                    inferred_type
                        .clone()
                        .map(|yt| yt.to_string())
                        .unwrap_or("unknown".to_string())
                )),
                ExprLiteral::String(x) => tree.add_leaf(&x),
                ExprLiteral::Bool(x) => tree.add_leaf(&x.to_string()),
            },
            //--------------------------------------------------------
            //is case
            Expr::Case(_) => todo!(),

            //--------------------------------------------------------
            //is switch
            Expr::Switch(ExprSwitch {
                inferred_type,
                cases,
                default_case,
                expr,
            }) => {
                let _branch = tree.add_branch(&format!("switch"));
                expr.add_to_tree(tree);
                for case in cases {
                    let _branch = tree.add_branch(&format!("case"));
                    Expr::Block(case.block.clone()).add_to_tree(tree);
                }
            }

            //--------------------------------------------------------
            //is function call
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                inferred_return_types,
                inferred_param_types,
                exprs,
            }) => {
                let _branch = tree.add_branch(&format!(
                    "{}({}): {}",
                    &function_name.to_string(),
                    format_inferred_types(&inferred_param_types),
                    format_inferred_types(&inferred_return_types)
                ));
                for expression in exprs.clone().into_iter() {
                    expression.add_to_tree(tree);
                }
            }

            //--------------------------------------------------------
            //is if statement
            Expr::IfStatement(ExprIfStatement {
                first_expr,
                second_expr,
            }) => {
                let _branch = tree.add_branch("if statement");
                let _conditional_branch = tree.add_branch("conditional");
                first_expr.add_to_tree(tree);
                let block = *second_expr.clone();
                Expr::Block(block).add_to_tree(tree);
            }

            //--------------------------------------------------------
            // is expr assignment
            Expr::Assignment(ExprAssignment {
                rhs,
                inferred_types,
                identifiers,
            }) => {
                let _branch = tree.add_branch(&format!(
                    "assign - {}",
                    identifiers
                        .iter()
                        .zip(inferred_types.iter())
                        .map(|(ident, yt)| format!("{}:{}", ident, inferred_type_to_string(yt)))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                rhs.add_to_tree(tree);
            }

            //--------------------------------------------------------
            //is declare variable
            Expr::DeclareVariable(ExprDeclareVariable {
                typed_identifiers,
                rhs,
            }) => {
                let _branch = tree.add_branch(&format!(
                    "declare - {}",
                    &typed_identifiers
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
                if let Some(rhs) = rhs {
                    rhs.add_to_tree(tree);
                }
            }

            //--------------------------------------------------------
            //is repeat
            Expr::Repeat(ExprRepeat {
                interior_block,
                iterations,
            }) => {
                let _branch = tree.add_branch(&format!("repeat {}", iterations));
                {
                    let _after_branch = tree.add_branch("interior block");
                    let block = *interior_block.clone();
                    Expr::Block(block).add_to_tree(tree);
                }
            }

            //--------------------------------------------------------
            //is for loop
            Expr::ForLoop(ExprForLoop {
                init_block,
                conditional,
                after_block,
                interior_block,
            }) => {
                let _branch = tree.add_branch("for loop");
                {
                    let _init_branch = tree.add_branch("init block");
                    let block = *init_block.clone();
                    Expr::Block(block).add_to_tree(tree);
                }
                {
                    let _conditional_branch = tree.add_branch("conditional");
                    conditional.add_to_tree(tree);
                }
                {
                    let _after_branch = tree.add_branch("after block");
                    let block = *after_block.clone();
                    Expr::Block(block).add_to_tree(tree);
                }
                {
                    let _after_branch = tree.add_branch("interior block");
                    let block = *interior_block.clone();
                    Expr::Block(block).add_to_tree(tree);
                }
            }

            //--------------------------------------------------------
            //is block
            Expr::Block(ExprBlock { exprs }) => {
                for expr in exprs {
                    expr.add_to_tree(tree);
                }
            }

            //--------------------------------------------------------
            //is variable
            Expr::Variable(ExprVariableReference {
                identifier,
                inferred_type,
            }) => {
                let _branch = tree.add_branch(&format!(
                    "var - {}:{}",
                    identifier,
                    inferred_type_to_string(inferred_type)
                ));
            }

            //--------------------------------------------------------
            //is function definition
            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name,
                params: typed_identifier_list,
                returns: return_typed_identifier_list,
                block,
            }) => {
                let _branch = tree.add_branch(&format!("function definition - {}", function_name));
                {
                    let _params_branch = tree.add_branch(&format!("params"));
                    for param in typed_identifier_list {
                        tree.add_leaf(&param.to_string());
                    }
                }
                {
                    let _params_branch = tree.add_branch(&format!("returns"));
                    for param in return_typed_identifier_list {
                        dbg!(&return_typed_identifier_list);
                        tree.add_leaf(&param.to_string());
                    }
                }
                let _branch = tree.add_branch(&format!("body"));
                Expr::Block(block.clone()).add_to_tree(tree);
            }

            //--------------------------------------------------------
            //is break
            Expr::Break => tree.add_leaf("break"),

            //--------------------------------------------------------
            //is continue
            Expr::Continue => tree.add_leaf("continue"),

            //--------------------------------------------------------
            //is leave
            Expr::Leave => tree.add_leaf("leave"),
        }
    }
}

//Add a Vec of Expr to an abstract syntax tree and return the tree as a string
pub fn expressions_to_tree(expressions: &Vec<Expr>) -> String {
    let mut tree = TreeBuilder::new();
    let _branch = tree.add_branch("AST");
    for expr in expressions {
        expr.add_to_tree(&mut tree);
    }
    tree.string()
}

//Implementations for Typed Identifier
impl fmt::Display for TypedIdentifier {
    //Print TypedIdentifier
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.identifier, self.yul_type)
    }
}

//Implementations for YulType
impl fmt::Display for YulType {
    //Print YulType
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YulType::U32 => write!(f, "u32"),
            YulType::U256 => write!(f, "u256"),
        }
    }
}

//Convert YulType to string
fn inferred_type_to_string(inferred_type: &Option<YulType>) -> String {
    match inferred_type {
        Some(yul_type) => yul_type.to_string(),
        None => "unknown".to_string(),
    }
}

//Convert a Vec of YulType to a string
fn format_inferred_types(inferred_types: &Vec<Option<YulType>>) -> String {
    inferred_types
        .iter()
        .map(|it| inferred_type_to_string(it))
        .collect::<Vec<_>>()
        .join(", ")
}
