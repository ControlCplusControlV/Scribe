// AST stuff

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Literal(u128),
    FunctionCall(ExprFunctionCall),
    Gt(ExprGt),
    Lt(ExprLt),
    DeclareVariable(ExprDeclareVariable),
    Variable(ExprVariableReference),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprVariableReference {
    pub identifier: String,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprDeclareVariable {
    pub identifier: String,
    pub rhs: Option<Box<Expr>>,
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
