// AST stuff

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Expr {
    Literal(u128),
    Add(ExprAdd),
    Gt(ExprGt),
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
    pub rhs: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprAdd {
    pub first_expr: Box<Expr>,
    pub second_expr: Box<Expr>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ExprGt {
    pub first_expr: Box<Expr>,
    pub second_expr: Box<Expr>,
}
