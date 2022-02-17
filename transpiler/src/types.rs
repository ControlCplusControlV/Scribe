use std::fmt;

use primitive_types::U256;

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
    Default(ExprDefault),
    Variable(ExprVariableReference),
    // Intermediate-AST-only expressions
    Repeat(ExprRepeat),
    Break,
    Continue,
    Leave,
    // //literals
    // Number(U256),
    // String(String),
    // Bool(bool),
}

impl Expr {
    pub fn get_inferred_type() -> Option<YulType> {
        todo!()
    }
}

#[derive(Hash, Clone, PartialEq, Eq, Debug, Copy)]
pub enum YulType {
    U32,
    U256,
}

impl YulType {
    pub fn from_annotation(annotation: &str) -> Self {
        match annotation {
            "u32" => Self::U32,
            "u256" => Self::U256,
            _ => panic!(),
        }
    }
    pub fn stack_width(&self) -> u32 {
        match self {
            Self::U32 => 1,
            Self::U256 => 8,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ExprLiteral {
    Number(ExprLiteralNumber),
    String(String),
    Bool(bool),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprLiteralNumber {
    pub inferred_type: Option<YulType>,
    // Always parse as u256, even if u32
    pub value: U256,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprVariableReference {
    pub identifier: String,
    pub inferred_type: Option<YulType>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprSwitch {
    pub default_case: Option<ExprBlock>,
    pub inferred_type: Option<YulType>,
    pub expr: Box<Expr>,
    pub cases: Vec<ExprCase>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprCase {
    pub literal: ExprLiteral,
    pub block: ExprBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprDefault {
    pub block: ExprBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprFunctionDefinition {
    pub function_name: String,
    pub params: Vec<TypedIdentifier>,
    pub returns: Vec<TypedIdentifier>,
    pub block: ExprBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBlock {
    pub exprs: Vec<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprAssignment {
    pub identifiers: Vec<String>,
    pub inferred_types: Vec<Option<YulType>>,
    pub rhs: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBreakContinue {
    pub identifier: String,
    pub rhs: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprForLoop {
    pub init_block: Box<ExprBlock>,
    pub conditional: Box<Expr>,
    pub after_block: Box<ExprBlock>,
    pub interior_block: Box<ExprBlock>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprRepeat {
    pub interior_block: Box<ExprBlock>,
    pub iterations: u32,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprDeclareVariable {
    pub typed_identifiers: Vec<TypedIdentifier>,
    pub rhs: Option<Box<Expr>>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprIfStatement {
    pub first_expr: Box<Expr>,
    pub second_expr: Box<ExprBlock>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprFunctionCall {
    pub function_name: String,
    pub exprs: Box<Vec<Expr>>,
    pub inferred_return_types: Vec<Option<YulType>>,
    pub inferred_param_types: Vec<Option<YulType>>,
}

#[derive(Hash, PartialEq, Eq, Debug, Clone)]
pub struct TypedIdentifier {
    pub identifier: String,
    pub yul_type: YulType,
}

pub type Identifier = String;

use debug_tree::{add_branch_to, add_leaf_to, TreeBuilder, TreeSymbols};

impl Expr {
    fn add_to_tree(&self, tree: &mut TreeBuilder) {
        match self {
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
            Expr::Case(_) => todo!(),
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

            // //TODO: Add case to tree
            // //is case
            // Expr::Case(ExprCase { literal, block }) => {}

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

            //is block
            Expr::Block(ExprBlock { exprs }) => {
                for expr in exprs {
                    expr.add_to_tree(tree);
                }
            }

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

            //is function definition
            //TODO: Add function Definition to tree
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

            //is break
            //TODO: add body
            Expr::Break => tree.add_leaf("break"),
            Expr::Continue => tree.add_leaf("continue"),
            Expr::Leave => tree.add_leaf("leave"),

            //is default
            //TODO: add body
            Expr::Default(ExprDefault { block }) => {}
        }
    }
}

pub fn expressions_to_tree(expressions: &Vec<Expr>) -> String {
    let mut tree = TreeBuilder::new();
    let _branch = tree.add_branch("AST");
    for expr in expressions {
        expr.add_to_tree(&mut tree);
    }
    tree.string()
}

impl fmt::Display for TypedIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.identifier, self.yul_type)
    }
}

impl fmt::Display for YulType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YulType::U32 => write!(f, "u32"),
            YulType::U256 => write!(f, "u256"),
        }
    }
}

fn inferred_type_to_string(inferred_type: &Option<YulType>) -> String {
    match inferred_type {
        Some(yul_type) => yul_type.to_string(),
        None => "unknown".to_string(),
    }
}

fn format_inferred_types(inferred_types: &Vec<Option<YulType>>) -> String {
    inferred_types
        .iter()
        .map(|it| inferred_type_to_string(it))
        .collect::<Vec<_>>()
        .join(", ")
}
