#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Literal(u32),
    FunctionDefinition(ExprFunctionDefinition),
    FunctionCall(ExprFunctionCall),
    IfStatement(ExprIfStatement),
    Assignment(ExprAssignment),
    DeclareVariable(ExprDeclareVariable),
    ForLoop(ExprForLoop),
    Block(ExprBlock),
    Break(ExprBreak),
    Continue(ExprContinue),
    Variable(ExprVariableReference),
    // Intermediate-AST-only expressions
    Repeat(ExprRepeat),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprVariableReference {
    pub identifier: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBreak {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprContinue {}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprFunctionDefinition {
    pub function_name: String,
    pub typed_identifier_list: Vec<Expr>,
    pub return_typed_identifier_list: Vec<Expr>,
    pub block: ExprBlock,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBlock {
    pub exprs: Vec<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprAssignment {
    pub identifier: String,
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
    pub identifier: String,
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
}

use debug_tree::{add_branch_to, add_leaf_to, TreeBuilder, TreeSymbols};

impl Expr {
    fn add_to_tree(&self, tree: &mut TreeBuilder) {
        match self {
            //is literal
            Expr::Literal(x) => tree.add_leaf(&x.to_string()),

            //is function call
            Expr::FunctionCall(ExprFunctionCall {
                function_name,
                exprs,
            }) => {
                let _branch = tree.add_branch(&format!("{}()", &function_name.to_string()));
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
            Expr::Assignment(ExprAssignment { identifier, rhs }) => {
                let _branch = tree.add_branch(&format!("assign - {}", &identifier.to_string()));
                rhs.add_to_tree(tree);
            }

            //is declare variable
            Expr::DeclareVariable(ExprDeclareVariable { identifier, rhs }) => {
                let _branch = tree.add_branch(&format!("declare - {}", &identifier.to_string()));
                if let Some(rhs) = rhs {
                    rhs.add_to_tree(tree);
                }
            }

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
            Expr::Variable(ExprVariableReference { identifier }) => {
                let _branch = tree.add_branch(&format!("var - {}", identifier));
            }

            //is function definition
            //TODO: Add function Definition to tree
            Expr::FunctionDefinition(ExprFunctionDefinition {
                function_name,
                typed_identifier_list,
                return_typed_identifier_list,
                block,
            }) => {}

            //is break
            Expr::Break(ExprBreak {}) => {}

            //is continue
            Expr::Continue(ExprContinue {}) => {}
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

#[test]
fn test_tree_printing() {
    let expressions = vec![Expr::FunctionCall(ExprFunctionCall {
        function_name: "add".to_string(),
        exprs: Box::new(vec![Expr::Literal(1), Expr::Literal(2)]),
    })];
    println!("{}", expressions_to_tree(&expressions));
}
