use super::asm;
use std::collections::HashMap;

pub fn fixup(instructions: &mut Vec<asm::Instruction>) {
    let stack_size = replace_pseudos(instructions);

    fix_moves(instructions);
    fix_div_imm(instructions);
    fix_binary(instructions);
    fix_shifts(instructions);
    fix_multiply(instructions);
    fix_compares(instructions);

    let aligned = (stack_size + 15) & !15;
    instructions.insert(0, asm::Instruction::AllocateStack(aligned));
}

fn replace_pseudos(instructions: &mut Vec<asm::Instruction>) -> i32 {
    let mut stack_map: HashMap<String, i32> = HashMap::new();
    let mut next_stack: i32 = 4;

    for inst in instructions.iter_mut() {
        match inst {
            asm::Instruction::Move { dst, src } => {
                replace_operand(dst, &mut stack_map, &mut next_stack);
                replace_operand(src, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Unary(_, op) => {
                replace_operand(op, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Binary(_, src1, src2) => {
                replace_operand(src1, &mut stack_map, &mut next_stack);
                replace_operand(src2, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Division(op) => {
                replace_operand(op, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::Compare(dst, src) => {
                replace_operand(dst, &mut stack_map, &mut next_stack);
                replace_operand(src, &mut stack_map, &mut next_stack);
            }

            asm::Instruction::SetCondition(_, dst) => {
                replace_operand(dst, &mut stack_map, &mut next_stack);
            }

            _ => {}
        }
    }

    next_stack - 4
}

fn replace_operand(
    op: &mut asm::Operand,
    stack_map: &mut HashMap<String, i32>,
    next_stack: &mut i32,
) {
    if let asm::Operand::Pseudo(name) = op {
        let offset = if let Some(existing) = stack_map.get(name) {
            *existing
        } else {
            let offset = *next_stack;
            stack_map.insert(name.clone(), offset);
            *next_stack += 4;
            offset
        };

        *op = asm::Operand::Stack(offset);
    }
}

fn fix_moves(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix = if let asm::Instruction::Move { dst, src } = &instructions[i] {
            is_stack_to_stack(dst, src)
        } else {
            false
        };

        if needs_fix {
            let (dst, src) = match &instructions[i] {
                asm::Instruction::Move { dst, src } => (dst.clone(), src.clone()),
                _ => unreachable!(),
            };

            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src,
            };

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

            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src: asm::Operand::Immediate(immediate),
            };

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

            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src,
            };

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

            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::CX),
                src,
            };

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

            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R11),
                src: dst.clone(),
            };

            instructions.insert(
                i + 1,
                asm::Instruction::Binary(
                    asm::BinaryOperator::Mul,
                    asm::Operand::Register(asm::Reg::R11),
                    src,
                ),
            );

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

fn fix_compares(instructions: &mut Vec<asm::Instruction>) {
    let mut i = 0;
    while i < instructions.len() {
        let needs_fix = if let asm::Instruction::Compare(dst, src) = &instructions[i] {
            is_stack_to_stack(dst, src)
        } else {
            false
        };

        if needs_fix {
            let (dst, src) = match &instructions[i] {
                asm::Instruction::Compare(dst, src) => (dst.clone(), src.clone()),
                _ => unreachable!(),
            };

            instructions[i] = asm::Instruction::Move {
                dst: asm::Operand::Register(asm::Reg::R10),
                src,
            };

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

fn is_stack_to_stack(dst: &asm::Operand, src: &asm::Operand) -> bool {
    matches!(dst, asm::Operand::Stack(_)) && matches!(src, asm::Operand::Stack(_))
}
