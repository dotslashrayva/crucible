// AST (Abstract Syntax Tree) structures
// These represent the structure of our program after parsing

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Block,
}

#[derive(Debug)]
pub struct Block {
    pub items: Vec<BlockItem>,
}

#[derive(Debug)]
pub enum BlockItem {
    Declaration(Declaration),
    Statement(Statement),
}

#[derive(Debug)]
pub struct Declaration {
    pub name: String,
    pub init: Option<Expr>,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expr),
    Expression(Expr),

    If(Expr, Box<Statement>, Option<Box<Statement>>),
    Compound(Block),

    Break(String),
    Continue(String),

    While(Expr, Box<Statement>, String),
    DoWhile(Box<Statement>, Expr, String),
    For(ForInit, Option<Expr>, Option<Expr>, Box<Statement>, String),

    Null,
}

#[derive(Debug)]
pub enum ForInit {
    InitDecl(Declaration),
    InitExpr(Option<Expr>),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Constant(i32),
    Variable(String),

    Unary(UnaryOperator, Box<Expr>),
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),

    Assignment(Box<Expr>, Box<Expr>),
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
    Complement,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,

    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,

    LeftShift,
    RightShift,

    LogicalAnd,
    LogicalOr,

    Equal,
    NotEqual,

    LessThan,
    LessOrEqual,

    GreaterThan,
    GreaterOrEqual,
}
