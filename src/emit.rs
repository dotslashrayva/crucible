use crate::asm::*;
use std::fmt::Write;

pub fn emit(program: Program) -> String {
    let mut output = String::new();
    emit_program(&program, &mut output);
    return output;
}

fn emit_program(program: &Program, output: &mut String) {
    writeln!(output, "\t.intel_syntax noprefix").unwrap();
    emit_function(&program.function, output);
}

fn emit_function(function: &Function, output: &mut String) {
    writeln!(output, "\t.globl _{}", function.name).unwrap();
    writeln!(output, "_{}:", function.name).unwrap();

    writeln!(output, "\tpush rbp").unwrap();
    writeln!(output, "\tmov rbp, rsp").unwrap();

    for instruction in &function.instructions {
        emit_instruction(instruction, output);
    }
}

fn emit_instruction(instruction: &Instruction, output: &mut String) {
    write!(output, "\t").unwrap();

    match instruction {
        Instruction::Move { dst, src } => {
            writeln!(output, "mov {}, {}", emit_operand(dst), emit_operand(src)).unwrap();
        }

        Instruction::Return => {
            writeln!(output).unwrap();
            writeln!(output, "\tmov rsp, rbp").unwrap();
            writeln!(output, "\tpop rbp").unwrap();
            writeln!(output, "\tret").unwrap();
        }

        Instruction::Unary(unop, oper) => match unop {
            UnaryOperator::Not => writeln!(output, "not {}", emit_operand(oper)).unwrap(),
            UnaryOperator::Neg => writeln!(output, "neg {}", emit_operand(oper)).unwrap(),
        },

        Instruction::AllocateStack(bytes) => {
            writeln!(output, "sub rsp, {}", bytes).unwrap();
            writeln!(output).unwrap();
        }

        Instruction::Binary(op, dst, src) => match op {
            BinaryOperator::Add => {
                writeln!(output, "add {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            BinaryOperator::Sub => {
                writeln!(output, "sub {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            BinaryOperator::Mul => {
                writeln!(output, "imul {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            BinaryOperator::And => {
                writeln!(output, "and {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            BinaryOperator::Or => {
                writeln!(output, "or {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            BinaryOperator::Xor => {
                writeln!(output, "xor {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
            }
            BinaryOperator::Sal => writeln!(
                output,
                "sal {}, {}",
                emit_operand(dst),
                emit_shift_count(src)
            )
            .unwrap(),
            BinaryOperator::Sar => writeln!(
                output,
                "sar {}, {}",
                emit_operand(dst),
                emit_shift_count(src)
            )
            .unwrap(),
        },

        Instruction::Division(divisor) => {
            writeln!(output, "idiv {}", emit_operand(divisor)).unwrap()
        }

        Instruction::ConvertDQ => writeln!(output, "cdq").unwrap(),

        Instruction::Compare(dst, src) => {
            writeln!(output, "cmp {}, {}", emit_operand(dst), emit_operand(src)).unwrap()
        }

        Instruction::Jump(label) => writeln!(output, "jmp L{}", label).unwrap(),

        Instruction::JumpCondition(condition, label) => {
            writeln!(output, "j{} L{}", emit_condition(condition), label).unwrap()
        }

        Instruction::SetCondition(condition, dst) => {
            writeln!(
                output,
                "set{} {}",
                emit_condition(condition),
                emit_one_byte_operand(dst)
            )
            .unwrap();
        }

        Instruction::Label(label) => {
            writeln!(output).unwrap();
            writeln!(output, "L{}:", label).unwrap()
        }
    }
}

fn emit_operand(operand: &Operand) -> String {
    match operand {
        Operand::Immediate(value) => value.to_string(),

        Operand::Register(reg) => match reg {
            Reg::AX => "eax",
            Reg::CX => "ecx",
            Reg::DX => "edx",
            Reg::R10 => "r10d",
            Reg::R11 => "r11d",
        }
        .to_string(),

        Operand::Stack(value) => format!("dword ptr [rbp - {}]", value),
        Operand::Pseudo(_value) => unreachable!(),
    }
}

fn emit_one_byte_operand(operand: &Operand) -> String {
    match operand {
        Operand::Immediate(value) => value.to_string(),
        Operand::Register(reg) => match reg {
            Reg::AX => "al",
            Reg::CX => "cl",
            Reg::DX => "dl",
            Reg::R10 => "r10b",
            Reg::R11 => "r11b",
        }
        .to_string(),
        Operand::Stack(value) => format!("byte ptr [rbp - {}]", value),
        Operand::Pseudo(_value) => unreachable!(),
    }
}

fn emit_shift_count(operand: &Operand) -> String {
    match operand {
        Operand::Immediate(value) => value.to_string(),
        Operand::Register(Reg::CX) => "cl".to_string(),
        _ => unreachable!("shift count must be immediate or cl"),
    }
}

fn emit_condition(condition: &Condition) -> String {
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
