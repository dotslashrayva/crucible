// Assembly program data structures
// These represent the assembly code we'll generate from the AST

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[allow(unused)]
#[derive(Debug)]
pub enum Instruction {
    Move { dst: Operand, src: Operand },
    Unary(UnaryOperator, Operand),
    Binary(BinaryOperator, Operand, Operand),
    Compare(Operand, Operand),
    Jump(String),
    JumpCondition(Condition, String),
    SetCondition(Condition, Operand),
    Label(String),
    Division(Operand),
    ConvertDQ,
    AllocateStack(i32),
    Return,
}

#[derive(Debug)]
pub enum UnaryOperator {
    Not,
    Neg,
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    Add,
    Sub,
    Mul,
    And,
    Or,
    Xor,
    Sal,
    Sar,
}

#[derive(Debug)]
pub enum Condition {
    Equal,
    NotEqual,

    Greater,
    GreaterEqual,

    Less,
    LessEqual,
}

#[derive(Debug, Clone)]
pub enum Operand {
    Immediate(i32),
    Register(Reg),
    Pseudo(String),
    Stack(i32),
}

#[derive(Debug, Clone)]
pub enum Reg {
    AX,
    DX,
    R10,
    R11,
}
