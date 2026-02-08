// Three-Address Code Intermediate Representation
// This IR is closer to assembly but still architecture-independent

#[derive(Debug)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub body: Vec<Instruction>,
}

#[allow(unused)]
#[derive(Debug)]
pub enum Instruction {
    Return(Value),
    Unary {
        op: UnaryOperator,
        dst: String,
        src: Value,
    },
    Binary {
        op: BinaryOperator,
        dst: String,
        src1: Value,
        src2: Value,
    },
    Copy {
        src: Value,
        dst: String,
    },
    Jump {
        target: String,
    },
    JumpIfZero {
        condition: Value,
        target: String,
    },
    JumpIfNotZero {
        condition: Value,
        target: String,
    },
    Label(String),
}

#[derive(Debug)]
pub enum Value {
    Constant(i32),    // like 8
    Variable(String), // like tmp.0
}

#[derive(Debug)]
pub enum UnaryOperator {
    Complement,
    Negate,
    Not,
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

    Equal,
    NotEqual,

    LessThan,
    LessOrEqual,

    GreaterThan,
    GreaterOrEqual,
}
