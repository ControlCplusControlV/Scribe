// AST stuff

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Literal(u128),
    FunctionCall(ExprFunctionCall),
    IfStatement(ExprIfStatement),
    Gt(ExprGt),
    Lt(ExprLt),
    DeclareVariable(ExprDeclareVariable),
    ForLoop(ExprForLoop),
    Block(ExprBlock),
    Variable(ExprVariableReference),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprVariableReference {
    pub identifier: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprBlock {
    pub exprs: Vec<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprForLoop {
    pub init_block: Box<ExprBlock>,
    pub conditional: Box<Expr>,
    pub after_block: Box<ExprBlock>,
    pub interior_block: Box<ExprBlock>,
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
    pub first_expr: Box<Expr>,
    pub second_expr: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprGt {
    pub first_expr: Box<Expr>,
    pub second_expr: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprLt {
    pub first_expr: Box<Expr>,
    pub second_expr: Box<Expr>,
}
