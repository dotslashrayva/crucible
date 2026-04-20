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

    If {
        condition: Expr,
        then_branch: Box<Statement>,
        else_branch: Option<Box<Statement>>,
    },

    Compound(Block),

    Break(String),
    Continue(String),

    While {
        condition: Expr,
        body: Box<Statement>,
        label: String,
    },

    DoWhile {
        body: Box<Statement>,
        condition: Expr,
        label: String,
    },

    For {
        init: ForInit,
        condition: Option<Expr>,
        post: Option<Expr>,
        body: Box<Statement>,
        label: String,
    },

    Goto(String),
    Labeled(String, Box<Statement>),

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

    Binary {
        op: BinaryOperator,
        left: Box<Expr>,
        right: Box<Expr>,
    },

    PostfixIncrement(Box<Expr>),
    PostfixDecrement(Box<Expr>),

    Assignment {
        target: Box<Expr>,
        value: Box<Expr>,
    },

    Conditional {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },

    CompoundAssignment {
        target: Box<Expr>,
        op: BinaryOperator,
        value: Box<Expr>,
    },
}

#[derive(Debug, Clone)]
pub enum UnaryOperator {
    Negate,
    LogicalNot,
    Complement,
    PrefixIncrement,
    PrefixDecrement,
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
