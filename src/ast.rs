// AST (Abstract Syntax Tree) structures
// These represent the structure of our program after parsing

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Statement,
}

#[derive(Debug)]
pub enum Statement {
    Return(Expr),
}

#[derive(Debug)]
pub enum Expr {
    Constant(i32),
    Unary(UnaryOperator, Box<Expr>),
    Binary(BinaryOperator, Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
    Complement,
}

#[derive(Debug)]
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
