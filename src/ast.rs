// AST (Abstract Syntax Tree) structures
// These represent the structure of our program after parsing

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Block>,
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
    Null,
}

#[derive(Debug)]
pub enum Block {
    State(Statement),
    Declare(Declaration),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Constant(i32),
    Variable(String),
    Unary(UnaryOperator, Box<Expr>),
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),
    Assignment(Box<Expr>, Box<Expr>),
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
