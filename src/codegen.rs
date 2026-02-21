use crate::asm;
use crate::ir;
use std::collections::HashMap;

pub fn generate(ir_program: ir::Program) -> asm::Program {
    let function = generate_function(ir_program.function);
    return asm::Program { function };
}

fn generate_function(ir_func: ir::Function) -> asm::Function {
    let name = ir_func.name;
    let mut instructions = generate_instruction(ir_func.body);

    let mut stack_map: HashMap<String, i32> = HashMap::new();
    let mut next_stack: i32 = 4;

    // Fix Pseudos
    for inst in &mut instructions {
        match inst {
            asm::Instruction::Move { dst, src } => {
                fix_operand(dst, &mut stack_map, &mut next_stack);
                fix_operand(src, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Unary(_, op) => {
                fix_operand(op, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Binary(_, src1, src2) => {
                fix_operand(src1, &mut stack_map, &mut next_stack);
                fix_operand(src2, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Division(op) => {
                fix_operand(op, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Compare(dst, src) => {
                fix_operand(dst, &mut stack_map, &mut next_stack);
                fix_operand(src, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::SetCondition(_, dst) => {
                fix_operand(dst, &mut stack_map, &mut next_stack)
            }

            _ => {}
        }
    }

    fix_moves(&mut instructions);
    fix_div_imm(&mut instructions);
    fix_binary(&mut instructions);
    fix_shifts(&mut instructions);
    fix_multiply(&mut instructions);
    fix_compares(&mut instructions);

    let stack_size = next_stack - 4;
    let aligned = (stack_size + 15) & !15;
    instructions.insert(0, asm::Instruction::AllocateStack(aligned));

    return asm::Function { name, instructions };
}

fn fix_multiply(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix =
            if let asm::Instruction::Binary(asm::BinaryOperator::Mul, dst, _) = &instructions[i] {
                matches!(dst, asm::Operand::Stack(_))
            } else {
                false
            };

        if needs_fix {
            let (dst, src) = match &instructions[i] {
                asm::Instruction::Binary(asm::BinaryOperator::Mul, dst, src) => {
                    (dst.clone(), src.clone())
                }
                _ => unreachable!(),
            };

            // mov r11, dst
            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R11),
                src: dst.clone(),
            };

            // imul src, r11
            instructions.insert(
                i + 1,
                asm::Instruction::Binary(
                    asm::BinaryOperator::Mul,
                    asm::Operand::Register(asm::Reg::R11),
                    src,
                ),
            );

            // mov dst, r11
            instructions.insert(
                i + 2,
                asm::Instruction::Move {
                    dst,
                    src: asm::Operand::Register(asm::Reg::R11),
                },
            );

            i += 3;
        } else {
            i += 1;
        }
    }
}

fn fix_div_imm(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix = if let asm::Instruction::Division(operand) = &instructions[i] {
            matches!(operand, asm::Operand::Immediate(_))
        } else {
            false
        };

        if needs_fix {
            let immediate = match &instructions[i] {
                asm::Instruction::Division(asm::Operand::Immediate(val)) => *val,
                _ => unreachable!(),
            };

            // mov r10, immediate
            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src: asm::Operand::Immediate(immediate),
            };

            // idiv r10
            instructions.insert(
                i + 1,
                asm::Instruction::Division(asm::Operand::Register(asm::Reg::R10)),
            );

            i += 2;
        } else {
            i += 1;
        }
    }
}

fn fix_binary(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix = if let asm::Instruction::Binary(_, op1, op2) = &instructions[i] {
            is_stack_to_stack(op1, op2)
        } else {
            false
        };

        if needs_fix {
            let (bin_op, dst, src) = match &instructions[i] {
                asm::Instruction::Binary(op, dst, src) => (op.clone(), dst.clone(), src.clone()),
                _ => unreachable!(),
            };

            // mov r10, src
            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src,
            };

            // binary dst, r10
            instructions.insert(
                i + 1,
                asm::Instruction::Binary(bin_op, dst, asm::Operand::Register(asm::Reg::R10)),
            );

            i += 2;
        } else {
            i += 1;
        }
    }
}

fn fix_shifts(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix = if let asm::Instruction::Binary(op, _, src) = &instructions[i] {
            matches!(op, asm::BinaryOperator::Sal | asm::BinaryOperator::Sar)
                && !matches!(src, asm::Operand::Immediate(_))
        } else {
            false
        };

        if needs_fix {
            let (bin_op, dst, src) = match &instructions[i] {
                asm::Instruction::Binary(op, dst, src) => (op.clone(), dst.clone(), src.clone()),
                _ => unreachable!(),
            };

            // mov ecx, <count>
            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::CX),
                src,
            };

            // sal/sar dst, cl
            instructions.insert(
                i + 1,
                asm::Instruction::Binary(bin_op, dst, asm::Operand::Register(asm::Reg::CX)),
            );

            i += 2;
        } else {
            i += 1;
        }
    }
}

fn fix_moves(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix;

        if let asm::Instruction::Move { dst, src } = &instructions[i] {
            needs_fix = is_stack_to_stack(dst, src);
        } else {
            needs_fix = false;
        }

        if needs_fix {
            let (dst, src) = match &instructions[i] {
                asm::Instruction::Move { dst, src } => (dst.clone(), src.clone()),
                _ => unreachable!(),
            };

            // mov r10, src
            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src,
            };

            // mov dst, r10
            instructions.insert(
                i + 1,
                asm::Instruction::Move {
                    dst,
                    src: asm::Operand::Register(asm::Reg::R10),
                },
            );

            i += 2;
        } else {
            i += 1;
        }
    }
}

fn fix_compares(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix;

        if let asm::Instruction::Compare(dst, src) = &instructions[i] {
            needs_fix = is_stack_to_stack(dst, src);
        } else {
            needs_fix = false;
        }

        if needs_fix {
            let (dst, src) = match &instructions[i] {
                asm::Instruction::Compare(dst, src) => (dst.clone(), src.clone()),
                _ => unreachable!(),
            };

            // mov r10, src
            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src,
            };

            // cmp dst, r10
            instructions.insert(
                i + 1,
                asm::Instruction::Compare(dst, asm::Operand::Register(asm::Reg::R10)),
            );

            i += 2;
        } else {
            i += 1;
        }
    }
}

fn fix_operand(op: &mut asm::Operand, stack_map: &mut HashMap<String, i32>, next_stack: &mut i32) {
    if let asm::Operand::Pseudo(name) = op {
        let offset;

        if let Some(existing) = stack_map.get(name) {
            offset = *existing;
        } else {
            offset = *next_stack;
            stack_map.insert(name.clone(), offset);
            *next_stack += 4; // one stack slot
        }

        *op = asm::Operand::Stack(offset);
    }
}

fn is_stack_to_stack(dst: &asm::Operand, src: &asm::Operand) -> bool {
    return matches!(dst, asm::Operand::Stack(_)) && matches!(src, asm::Operand::Stack(_));
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
