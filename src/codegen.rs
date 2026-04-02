use crate::asm;
use crate::fixup;
use crate::ir;

pub fn generate(ir_program: ir::Program) -> asm::Program {
    let function = generate_function(ir_program.function);
    return asm::Program { function };
}

fn generate_function(ir_func: ir::Function) -> asm::Function {
    let name = ir_func.name;
    let mut instructions = generate_instruction(ir_func.body);
    fixup::fixup(&mut instructions);
    return asm::Function { name, instructions };
}

fn map_src_operand(src: &ir::Value) -> asm::Operand {
    match src {
        ir::Value::Constant(val) => asm::Operand::Immediate(*val),
        ir::Value::Variable(var) => asm::Operand::Pseudo(var.clone()),
    }
}

fn map_unary(op: &ir::UnaryOperator) -> asm::UnaryOperator {
    match op {
        ir::UnaryOperator::Complement => asm::UnaryOperator::Not, // NOT
        ir::UnaryOperator::Negate => asm::UnaryOperator::Neg,     // NEG
        ir::UnaryOperator::Not => unreachable!(),
    }
}

fn map_binary(op: &ir::BinaryOperator) -> asm::BinaryOperator {
    match op {
        ir::BinaryOperator::Add => asm::BinaryOperator::Add,
        ir::BinaryOperator::Subtract => asm::BinaryOperator::Sub,
        ir::BinaryOperator::Multiply => asm::BinaryOperator::Mul,
        ir::BinaryOperator::BitwiseAnd => asm::BinaryOperator::And,
        ir::BinaryOperator::BitwiseOr => asm::BinaryOperator::Or,
        ir::BinaryOperator::BitwiseXor => asm::BinaryOperator::Xor,
        ir::BinaryOperator::LeftShift => asm::BinaryOperator::Sal,
        ir::BinaryOperator::RightShift => asm::BinaryOperator::Sar,
        _ => unreachable!(),
    }
}

fn map_binary_relational(op: &ir::BinaryOperator) -> asm::Condition {
    match op {
        ir::BinaryOperator::Equal => asm::Condition::Equal,
        ir::BinaryOperator::NotEqual => asm::Condition::NotEqual,

        ir::BinaryOperator::GreaterThan => asm::Condition::Greater,
        ir::BinaryOperator::GreaterOrEqual => asm::Condition::GreaterEqual,

        ir::BinaryOperator::LessThan => asm::Condition::Less,
        ir::BinaryOperator::LessOrEqual => asm::Condition::LessEqual,
        _ => unreachable!(),
    }
}

fn generate_instruction(instructions: Vec<ir::Instruction>) -> Vec<asm::Instruction> {
    let mut out: Vec<asm::Instruction> = Vec::new();

    for inst in &instructions {
        match inst {
            ir::Instruction::Return(value) => {
                match value {
                    ir::Value::Constant(val) => {
                        out.push(asm::Instruction::Move {
                            dst: asm::Operand::Register(asm::Reg::AX),
                            src: asm::Operand::Immediate(*val),
                        });
                    }

                    ir::Value::Variable(val) => {
                        out.push(asm::Instruction::Move {
                            dst: asm::Operand::Register(asm::Reg::AX),
                            src: asm::Operand::Pseudo(val.clone()),
                        });
                    }
                }

                out.push(asm::Instruction::Return);
            }

            ir::Instruction::Unary { op, src, dst } => match op {
                ir::UnaryOperator::Not => {
                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Register(asm::Reg::R11),
                        src: asm::Operand::Immediate(0),
                    });

                    out.push(asm::Instruction::Compare(
                        asm::Operand::Register(asm::Reg::R11),
                        map_src_operand(src),
                    ));

                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Pseudo(dst.clone()),
                        src: asm::Operand::Immediate(0),
                    });

                    out.push(asm::Instruction::SetCondition(
                        asm::Condition::Equal,
                        asm::Operand::Pseudo(dst.clone()),
                    ));
                }
                _ => {
                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Pseudo(dst.clone()),
                        src: map_src_operand(src),
                    });

                    out.push(asm::Instruction::Unary(
                        map_unary(&op),
                        asm::Operand::Pseudo(dst.clone()),
                    ));
                }
            },

            ir::Instruction::Binary {
                op,
                src1,
                src2,
                dst,
            } => match op {
                // Divide (/) and Modulo (%)
                ir::BinaryOperator::Divide | ir::BinaryOperator::Modulo => {
                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Register(asm::Reg::AX),
                        src: map_src_operand(src1),
                    });

                    out.push(asm::Instruction::ConvertDQ);
                    out.push(asm::Instruction::Division(map_src_operand(src2)));

                    let result_reg = if matches!(op, ir::BinaryOperator::Divide) {
                        asm::Reg::AX
                    } else {
                        asm::Reg::DX
                    };

                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Pseudo(dst.clone()),
                        src: asm::Operand::Register(result_reg),
                    });
                }

                // Eq, NotEq, Greater, Less. etc.
                ir::BinaryOperator::Equal
                | ir::BinaryOperator::NotEqual
                | ir::BinaryOperator::GreaterThan
                | ir::BinaryOperator::GreaterOrEqual
                | ir::BinaryOperator::LessThan
                | ir::BinaryOperator::LessOrEqual => {
                    match map_src_operand(src1) {
                        asm::Operand::Immediate(val) => {
                            out.push(asm::Instruction::Move {
                                dst: asm::Operand::Register(asm::Reg::R11),
                                src: asm::Operand::Immediate(val),
                            });

                            out.push(asm::Instruction::Compare(
                                asm::Operand::Register(asm::Reg::R11),
                                map_src_operand(src2),
                            ));
                        }
                        _ => {
                            out.push(asm::Instruction::Compare(
                                map_src_operand(src1),
                                map_src_operand(src2),
                            ));
                        }
                    }

                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Pseudo(dst.clone()),
                        src: asm::Operand::Immediate(0),
                    });

                    out.push(asm::Instruction::SetCondition(
                        map_binary_relational(op),
                        asm::Operand::Pseudo(dst.clone()),
                    ));
                }

                // Add, Sub, Mul, Bitwise
                _ => {
                    out.push(asm::Instruction::Move {
                        dst: asm::Operand::Pseudo(dst.clone()),
                        src: map_src_operand(src1),
                    });

                    out.push(asm::Instruction::Binary(
                        map_binary(op),
                        asm::Operand::Pseudo(dst.clone()),
                        map_src_operand(src2),
                    ));
                }
            },

            ir::Instruction::JumpIfZero { condition, target } => {
                out.push(asm::Instruction::Move {
                    dst: asm::Operand::Register(asm::Reg::R11),
                    src: asm::Operand::Immediate(0),
                });

                out.push(asm::Instruction::Compare(
                    asm::Operand::Register(asm::Reg::R11),
                    map_src_operand(condition),
                ));

                out.push(asm::Instruction::JumpCondition(
                    asm::Condition::Equal,
                    target.clone(),
                ));
            }

            ir::Instruction::JumpIfNotZero { condition, target } => {
                out.push(asm::Instruction::Move {
                    dst: asm::Operand::Register(asm::Reg::R11),
                    src: asm::Operand::Immediate(0),
                });

                out.push(asm::Instruction::Compare(
                    asm::Operand::Register(asm::Reg::R11),
                    map_src_operand(condition),
                ));

                out.push(asm::Instruction::JumpCondition(
                    asm::Condition::NotEqual,
                    target.clone(),
                ));
            }

            ir::Instruction::Copy { src, dst } => out.push(asm::Instruction::Move {
                dst: asm::Operand::Pseudo(dst.clone()),
                src: map_src_operand(src),
            }),

            ir::Instruction::Label(ident) => out.push(asm::Instruction::Label(ident.clone())),
            ir::Instruction::Jump { target } => out.push(asm::Instruction::Jump(target.clone())),
        }
    }

    return out;
}
