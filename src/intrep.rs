use crate::ast;
use crate::ir;

struct IntRep {
    instructions: Vec<ir::Instruction>,
    var_count: u32,
    label_count: u32,
}

impl IntRep {
    fn new() -> Self {
        Self {
            instructions: Vec::new(),
            var_count: 0,
            label_count: 0,
        }
    }

    fn alloc_var(&mut self) -> String {
        let name = format!("tmp.{}", self.var_count);
        self.var_count += 1;
        return name;
    }

    fn alloc_label(&mut self, prefix: &str) -> String {
        let name = format!("{}.{}", prefix, self.label_count);
        self.label_count += 1;
        return name;
    }

    fn append(&mut self, instr: ir::Instruction) {
        self.instructions.push(instr);
    }

    fn convert_unary_op(op: &ast::UnaryOperator) -> ir::UnaryOperator {
        match op {
            ast::UnaryOperator::Complement => ir::UnaryOperator::Complement,
            ast::UnaryOperator::Negate => ir::UnaryOperator::Negate,
            ast::UnaryOperator::LogicalNot => ir::UnaryOperator::Not,
        }
    }

    fn convert_binary_op(op: &ast::BinaryOperator) -> ir::BinaryOperator {
        match op {
            ast::BinaryOperator::Add => ir::BinaryOperator::Add,
            ast::BinaryOperator::Subtract => ir::BinaryOperator::Subtract,
            ast::BinaryOperator::Multiply => ir::BinaryOperator::Multiply,
            ast::BinaryOperator::Divide => ir::BinaryOperator::Divide,
            ast::BinaryOperator::Modulo => ir::BinaryOperator::Modulo,

            ast::BinaryOperator::BitwiseAnd => ir::BinaryOperator::BitwiseAnd,
            ast::BinaryOperator::BitwiseOr => ir::BinaryOperator::BitwiseOr,
            ast::BinaryOperator::BitwiseXor => ir::BinaryOperator::BitwiseXor,

            ast::BinaryOperator::LeftShift => ir::BinaryOperator::LeftShift,
            ast::BinaryOperator::RightShift => ir::BinaryOperator::RightShift,

            ast::BinaryOperator::Equal => ir::BinaryOperator::Equal,
            ast::BinaryOperator::NotEqual => ir::BinaryOperator::NotEqual,

            ast::BinaryOperator::LessThan => ir::BinaryOperator::LessThan,
            ast::BinaryOperator::LessOrEqual => ir::BinaryOperator::LessOrEqual,

            ast::BinaryOperator::GreaterThan => ir::BinaryOperator::GreaterThan,
            ast::BinaryOperator::GreaterOrEqual => ir::BinaryOperator::GreaterOrEqual,

            ast::BinaryOperator::LogicalAnd => unreachable!(),
            ast::BinaryOperator::LogicalOr => unreachable!(),
        }
    }
}

// Main IR function
pub fn flatten(ast_program: ast::Program) -> ir::Program {
    let function = flatten_function(ast_program.function);
    return ir::Program { function };
}

fn flatten_function(ast_func: ast::Function) -> ir::Function {
    let mut irctx = IntRep::new();

    flatten_statement(ast_func.body, &mut irctx);

    return ir::Function {
        name: ast_func.name,
        body: irctx.instructions,
    };
}

fn flatten_statement(statement: ast::Statement, irctx: &mut IntRep) {
    match statement {
        ast::Statement::Return(expr) => {
            let result_val = flatten_expr(expr, irctx);
            irctx.append(ir::Instruction::Return(result_val));
        }
    }
}

fn flatten_expr(expr: ast::Expr, irctx: &mut IntRep) -> ir::Value {
    match expr {
        ast::Expr::Constant(val) => return ir::Value::Constant(val),

        ast::Expr::Unary(op, inner) => {
            let src = flatten_expr(*inner, irctx);
            let dst = irctx.alloc_var();

            irctx.append(ir::Instruction::Unary {
                op: IntRep::convert_unary_op(&op),
                dst: dst.clone(),
                src: src,
            });

            return ir::Value::Variable(dst);
        }

        ast::Expr::Binary(op, left, right) => {
            match op {
                ast::BinaryOperator::LogicalAnd => {
                    // For: left && right
                    // If left is false (0), result is 0 without evaluating right
                    // If left is true (non-zero), result is (right != 0)

                    let result = irctx.alloc_var();
                    let false_label = irctx.alloc_label("and_false");
                    let end_label = irctx.alloc_label("and_end");

                    // Evaluate left operand
                    let v1 = flatten_expr(*left, irctx);

                    // If left is false (0), jump to false_label
                    irctx.append(ir::Instruction::JumpIfZero {
                        condition: v1,
                        target: false_label.clone(),
                    });

                    // Left is true, evaluate right operand
                    let v2 = flatten_expr(*right, irctx);

                    // Convert right to boolean (0 or 1)
                    let right_bool = irctx.alloc_var();
                    irctx.append(ir::Instruction::Binary {
                        op: ir::BinaryOperator::NotEqual,
                        src1: v2,
                        src2: ir::Value::Constant(0),
                        dst: right_bool.clone(),
                    });

                    // Store result and jump to end
                    irctx.append(ir::Instruction::Copy {
                        src: ir::Value::Variable(right_bool),
                        dst: result.clone(),
                    });
                    irctx.append(ir::Instruction::Jump {
                        target: end_label.clone(),
                    });

                    // False label: set result to 0
                    irctx.append(ir::Instruction::Label(false_label));
                    irctx.append(ir::Instruction::Copy {
                        src: ir::Value::Constant(0),
                        dst: result.clone(),
                    });

                    // End label
                    irctx.append(ir::Instruction::Label(end_label));

                    return ir::Value::Variable(result);
                }

                ast::BinaryOperator::LogicalOr => {
                    // For: left || right
                    // If left is true (non-zero), result is 1 without evaluating right
                    // If left is false (0), result is (right != 0)

                    let result = irctx.alloc_var();
                    let true_label = irctx.alloc_label("or_true");
                    let end_label = irctx.alloc_label("or_end");

                    // Evaluate left operand
                    let v1 = flatten_expr(*left, irctx);

                    // If left is true (non-zero), jump to true_label
                    irctx.append(ir::Instruction::JumpIfNotZero {
                        condition: v1,
                        target: true_label.clone(),
                    });

                    // Left is false, evaluate right operand
                    let v2 = flatten_expr(*right, irctx);

                    // Convert right to boolean (0 or 1)
                    let right_bool = irctx.alloc_var();
                    irctx.append(ir::Instruction::Binary {
                        op: ir::BinaryOperator::NotEqual,
                        src1: v2,
                        src2: ir::Value::Constant(0),
                        dst: right_bool.clone(),
                    });

                    // Store result and jump to end
                    irctx.append(ir::Instruction::Copy {
                        src: ir::Value::Variable(right_bool),
                        dst: result.clone(),
                    });
                    irctx.append(ir::Instruction::Jump {
                        target: end_label.clone(),
                    });

                    // True label: set result to 1
                    irctx.append(ir::Instruction::Label(true_label));
                    irctx.append(ir::Instruction::Copy {
                        src: ir::Value::Constant(1),
                        dst: result.clone(),
                    });

                    // End label
                    irctx.append(ir::Instruction::Label(end_label));

                    return ir::Value::Variable(result);
                }

                _ => {
                    let v1 = flatten_expr(*left, irctx);
                    let v2 = flatten_expr(*right, irctx);
                    let dst = irctx.alloc_var();

                    irctx.append(ir::Instruction::Binary {
                        op: IntRep::convert_binary_op(&op),
                        src1: v1,
                        src2: v2,
                        dst: dst.clone(),
                    });

                    return ir::Value::Variable(dst);
                }
            }
        }
    }
}
