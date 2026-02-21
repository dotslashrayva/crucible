use crate::asm::{self, Condition};
use std::fmt::Write;

pub fn emit(program: asm::Program) -> String {
    let mut output = String::new();
    emit_program(&program, &mut output);
    return output;
}

fn emit_program(program: &asm::Program, output: &mut String) {
    writeln!(output, "\t.intel_syntax noprefix").unwrap();
    emit_function(&program.function, output);
}

fn emit_function(function: &asm::Function, output: &mut String) {
    writeln!(output, "\t.globl _{}", function.name).unwrap();
    writeln!(output, "_{}:", function.name).unwrap();

    writeln!(output, "\tpush rbp").unwrap();
    writeln!(output, "\tmov rbp, rsp").unwrap();

    for instruction in &function.instructions {
        emit_instruction(instruction, output);
    }
}

fn emit_instruction(instruction: &asm::Instruction, output: &mut String) {
    write!(output, "\t").unwrap();

    match instruction {
        asm::Instruction::Move { dst, src } => {
            writeln!(output, "mov {}, {}", emit_operand(dst), emit_operand(src)).unwrap();
        }

        asm::Instruction::Return => {
            writeln!(output).unwrap();
            writeln!(output, "\tmov rsp, rbp").unwrap();
            writeln!(output, "\tpop rbp").unwrap();
            writeln!(output, "\tret").unwrap();
        }

        asm::Instruction::Unary(unop, oper) => match unop {
            asm::UnaryOperator::Not => writeln!(output, "not {}", emit_operand(oper)).unwrap(),
            asm::UnaryOperator::Neg => writeln!(output, "neg {}", emit_operand(oper)).unwrap(),
        },

        asm::Instruction::AllocateStack(bytes) => {
            writeln!(output, "sub rsp, {}", bytes).unwrap();
            writeln!(output).unwrap();
        }

        asm::Instruction::Binary(op, dst, src) => match op {
            asm::BinaryOperator::Add => {
                writeln!(output, "add {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            asm::BinaryOperator::Sub => {
                writeln!(output, "sub {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            asm::BinaryOperator::Mul => {
                writeln!(output, "imul {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            asm::BinaryOperator::And => {
                writeln!(output, "and {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            asm::BinaryOperator::Or => {
                writeln!(output, "or {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            asm::BinaryOperator::Xor => {
                writeln!(output, "xor {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            asm::BinaryOperator::Sal => writeln!(
                output,
                "sal {}, {}",
                emit_operand(dst),
                emit_shift_count(src)
            )
            .unwrap(),
            asm::BinaryOperator::Sar => writeln!(
                output,
                "sar {}, {}",
                emit_operand(dst),
                emit_shift_count(src)
            )
            .unwrap(),
        },

        asm::Instruction::Division(divisor) => {
            writeln!(output, "idiv {}", emit_operand(divisor)).unwrap()
        }

        asm::Instruction::ConvertDQ => writeln!(output, "cdq").unwrap(),

        asm::Instruction::Compare(dst, src) => {
            writeln!(output, "cmp {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
        }

        asm::Instruction::Jump(label) => writeln!(output, "jmp L{}", label).unwrap(),

        asm::Instruction::JumpCondition(condition, label) => {
            writeln!(output, "j{} L{}", emit_condition(condition), label).unwrap()
        }

        asm::Instruction::SetCondition(condition, dst) => {
            writeln!(
                output,
                "set{} {}",
                emit_condition(condition),
                emit_one_byte_operand(dst)
            )
            .unwrap();
        }

        asm::Instruction::Label(label) => {
            writeln!(output).unwrap();
            writeln!(output, "L{}:", label).unwrap()
        }
    }
}

fn emit_operand(operand: &asm::Operand) -> String {
    match operand {
        asm::Operand::Immediate(value) => value.to_string(),

        asm::Operand::Register(reg) => match reg {
            asm::Reg::AX => "eax",
            asm::Reg::CX => "ecx",
            asm::Reg::DX => "edx",
            asm::Reg::R10 => "r10d",
            asm::Reg::R11 => "r11d",
        }
        .to_string(),

        asm::Operand::Stack(value) => format!("dword ptr [rbp - {}]", value),
        asm::Operand::Pseudo(_value) => unreachable!(),
    }
}

fn emit_one_byte_operand(operand: &asm::Operand) -> String {
    match operand {
        asm::Operand::Immediate(value) => value.to_string(),
        asm::Operand::Register(reg) => match reg {
            asm::Reg::AX => "al",
            asm::Reg::CX => "cl",
            asm::Reg::DX => "dl",
            asm::Reg::R10 => "r10b",
            asm::Reg::R11 => "r11b",
        }
        .to_string(),
        asm::Operand::Stack(value) => format!("byte ptr [rbp - {}]", value),
        asm::Operand::Pseudo(_value) => unreachable!(),
    }
}

fn emit_shift_count(operand: &asm::Operand) -> String {
    match operand {
        asm::Operand::Immediate(value) => value.to_string(),
        asm::Operand::Register(asm::Reg::CX) => "cl".to_string(),
        _ => unreachable!("shift count must be immediate or cl"),
    }
}

fn emit_condition(condition: &asm::Condition) -> String {
    match condition {
        Condition::Equal => "e",
        Condition::NotEqual => "ne",

        Condition::Less => "l",
        Condition::LessEqual => "le",

        Condition::Greater => "g",
        Condition::GreaterEqual => "ge",
    }
    .to_string()
}
