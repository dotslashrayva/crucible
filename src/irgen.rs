use crate::ast;
use crate::ir;

struct Context {
    instructions: Vec<ir::Instruction>,
    var_count: u32,
    label_count: u32,
}

impl Context {
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

    fn break_label(loop_label: &str) -> String {
        format!("break_{}", loop_label)
    }

    fn continue_label(loop_label: &str) -> String {
        format!("continue_{}", loop_label)
    }

    fn convert_unary_op(op: &ast::UnaryOperator) -> ir::UnaryOperator {
        match op {
            ast::UnaryOperator::Complement => ir::UnaryOperator::Complement,
            ast::UnaryOperator::Negate => ir::UnaryOperator::Negate,
            ast::UnaryOperator::LogicalNot => ir::UnaryOperator::Not,
            ast::UnaryOperator::PrefixIncrement | ast::UnaryOperator::PrefixDecrement => {
                unreachable!("Prefix ++/-- handled separately in flatten_expr")
            }
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
    let mut ctx = Context::new();

    flatten_block(ast_func.body, &mut ctx);

    if !matches!(ctx.instructions.last(), Some(ir::Instruction::Return(_))) {
        ctx.append(ir::Instruction::Return(ir::Value::Constant(0)));
    }

    return ir::Function {
        name: ast_func.name,
        body: ctx.instructions,
    };
}

fn flatten_block(block: ast::Block, ctx: &mut Context) {
    for item in block.items {
        flatten_block_item(item, ctx);
    }
}

fn flatten_block_item(item: ast::BlockItem, ctx: &mut Context) {
    match item {
        ast::BlockItem::Declaration(decl) => flatten_declaration(decl, ctx),
        ast::BlockItem::Statement(stmt) => flatten_statement(stmt, ctx),
    }
}

fn flatten_declaration(decl: ast::Declaration, ctx: &mut Context) {
    if let Some(init) = decl.init {
        let val = flatten_expr(init, ctx);
        ctx.append(ir::Instruction::Copy {
            src: val,
            dst: decl.name,
        });
    }
}

fn flatten_for_init(init: ast::ForInit, ctx: &mut Context) {
    match init {
        ast::ForInit::InitDecl(decl) => flatten_declaration(decl, ctx),
        ast::ForInit::InitExpr(Some(expr)) => {
            flatten_expr(expr, ctx);
        }
        ast::ForInit::InitExpr(None) => {}
    }
}

fn flatten_statement(statement: ast::Statement, ctx: &mut Context) {
    match statement {
        ast::Statement::Return(expr) => {
            let result_val = flatten_expr(expr, ctx);
            ctx.append(ir::Instruction::Return(result_val));
        }

        ast::Statement::If(condition, then_stmt, else_stmt) => {
            let cond_val = flatten_expr(condition, ctx);

            match else_stmt {
                None => {
                    let end_label = ctx.alloc_label("if_end");

                    ctx.append(ir::Instruction::JumpIfZero {
                        condition: cond_val,
                        target: end_label.clone(),
                    });

                    flatten_statement(*then_stmt, ctx);

                    ctx.append(ir::Instruction::Label(end_label));
                }
                Some(else_stmt) => {
                    let else_label = ctx.alloc_label("if_else");
                    let end_label = ctx.alloc_label("if_end");

                    ctx.append(ir::Instruction::JumpIfZero {
                        condition: cond_val,
                        target: else_label.clone(),
                    });

                    flatten_statement(*then_stmt, ctx);

                    ctx.append(ir::Instruction::Jump {
                        target: end_label.clone(),
                    });

                    ctx.append(ir::Instruction::Label(else_label));

                    flatten_statement(*else_stmt, ctx);

                    ctx.append(ir::Instruction::Label(end_label));
                }
            }
        }

        ast::Statement::Expression(expr) => {
            flatten_expr(expr, ctx);
        }

        ast::Statement::Compound(block) => {
            flatten_block(block, ctx);
        }

        // break label; -> Jump(break_<label>)
        ast::Statement::Break(label) => {
            ctx.append(ir::Instruction::Jump {
                target: Context::break_label(&label),
            });
        }

        // continue label; -> Jump(continue_<label>)
        ast::Statement::Continue(label) => {
            ctx.append(ir::Instruction::Jump {
                target: Context::continue_label(&label),
            });
        }

        // While loop:
        ast::Statement::While(condition, body, label) => {
            let cont_label = Context::continue_label(&label);
            let brk_label = Context::break_label(&label);

            ctx.append(ir::Instruction::Label(cont_label.clone()));

            let cond_val = flatten_expr(condition, ctx);
            ctx.append(ir::Instruction::JumpIfZero {
                condition: cond_val,
                target: brk_label.clone(),
            });

            flatten_statement(*body, ctx);

            ctx.append(ir::Instruction::Jump { target: cont_label });

            ctx.append(ir::Instruction::Label(brk_label));
        }

        // Do-While loop:
        ast::Statement::DoWhile(body, condition, label) => {
            let start_label = ctx.alloc_label("do_start");
            let cont_label = Context::continue_label(&label);
            let brk_label = Context::break_label(&label);

            ctx.append(ir::Instruction::Label(start_label.clone()));

            flatten_statement(*body, ctx);

            ctx.append(ir::Instruction::Label(cont_label));

            let cond_val = flatten_expr(condition, ctx);
            ctx.append(ir::Instruction::JumpIfNotZero {
                condition: cond_val,
                target: start_label,
            });

            ctx.append(ir::Instruction::Label(brk_label));
        }

        // For loop:
        ast::Statement::For(init, condition, post, body, label) => {
            let start_label = ctx.alloc_label("for_start");
            let cont_label = Context::continue_label(&label);
            let brk_label = Context::break_label(&label);

            // Init clause
            flatten_for_init(init, ctx);

            ctx.append(ir::Instruction::Label(start_label.clone()));

            // Condition: if present, emit JumpIfZero; if absent, omit entirely
            if let Some(cond) = condition {
                let cond_val = flatten_expr(cond, ctx);
                ctx.append(ir::Instruction::JumpIfZero {
                    condition: cond_val,
                    target: brk_label.clone(),
                });
            }

            flatten_statement(*body, ctx);

            ctx.append(ir::Instruction::Label(cont_label));

            // Post expression
            if let Some(post_expr) = post {
                flatten_expr(post_expr, ctx);
            }

            ctx.append(ir::Instruction::Jump {
                target: start_label,
            });

            ctx.append(ir::Instruction::Label(brk_label));
        }

        ast::Statement::Null => {}
    }
}

fn flatten_expr(expr: ast::Expr, ctx: &mut Context) -> ir::Value {
    match expr {
        ast::Expr::Constant(val) => return ir::Value::Constant(val),

        // Prefix ++x: increment, return new value
        ast::Expr::Unary(ast::UnaryOperator::PrefixIncrement, inner) => {
            let var = match *inner {
                ast::Expr::Variable(name) => name,
                _ => unreachable!(),
            };

            let dst = ctx.alloc_var();
            ctx.append(ir::Instruction::Binary {
                op: ir::BinaryOperator::Add,
                src1: ir::Value::Variable(var.clone()),
                src2: ir::Value::Constant(1),
                dst: dst.clone(),
            });
            ctx.append(ir::Instruction::Copy {
                src: ir::Value::Variable(dst.clone()),
                dst: var,
            });

            ir::Value::Variable(dst)
        }

        // Prefix --x: decrement, return new value
        ast::Expr::Unary(ast::UnaryOperator::PrefixDecrement, inner) => {
            let var = match *inner {
                ast::Expr::Variable(name) => name,
                _ => unreachable!(),
            };

            let dst = ctx.alloc_var();
            ctx.append(ir::Instruction::Binary {
                op: ir::BinaryOperator::Subtract,
                src1: ir::Value::Variable(var.clone()),
                src2: ir::Value::Constant(1),
                dst: dst.clone(),
            });
            ctx.append(ir::Instruction::Copy {
                src: ir::Value::Variable(dst.clone()),
                dst: var,
            });

            ir::Value::Variable(dst)
        }

        // Postfix x++: increment, return old value
        ast::Expr::PostfixIncrement(inner) => {
            let var = match *inner {
                ast::Expr::Variable(name) => name,
                _ => unreachable!(),
            };

            let old = ctx.alloc_var();
            ctx.append(ir::Instruction::Copy {
                src: ir::Value::Variable(var.clone()),
                dst: old.clone(),
            });
            ctx.append(ir::Instruction::Binary {
                op: ir::BinaryOperator::Add,
                src1: ir::Value::Variable(var.clone()),
                src2: ir::Value::Constant(1),
                dst: var,
            });

            ir::Value::Variable(old)
        }

        // Postfix x--: decrement, return old value
        ast::Expr::PostfixDecrement(inner) => {
            let var = match *inner {
                ast::Expr::Variable(name) => name,
                _ => unreachable!(),
            };

            let old = ctx.alloc_var();
            ctx.append(ir::Instruction::Copy {
                src: ir::Value::Variable(var.clone()),
                dst: old.clone(),
            });
            ctx.append(ir::Instruction::Binary {
                op: ir::BinaryOperator::Subtract,
                src1: ir::Value::Variable(var.clone()),
                src2: ir::Value::Constant(1),
                dst: var,
            });

            ir::Value::Variable(old)
        }

        ast::Expr::Unary(op, inner) => {
            let src = flatten_expr(*inner, ctx);
            let dst = ctx.alloc_var();

            ctx.append(ir::Instruction::Unary {
                op: Context::convert_unary_op(&op),
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

                    let result = ctx.alloc_var();
                    let false_label = ctx.alloc_label("and_false");
                    let end_label = ctx.alloc_label("and_end");

                    // Evaluate left operand
                    let v1 = flatten_expr(*left, ctx);

                    // If left is false (0), jump to false_label
                    ctx.append(ir::Instruction::JumpIfZero {
                        condition: v1,
                        target: false_label.clone(),
                    });

                    // Left is true, evaluate right operand
                    let v2 = flatten_expr(*right, ctx);

                    // Convert right to boolean (0 or 1)
                    let right_bool = ctx.alloc_var();
                    ctx.append(ir::Instruction::Binary {
                        op: ir::BinaryOperator::NotEqual,
                        src1: v2,
                        src2: ir::Value::Constant(0),
                        dst: right_bool.clone(),
                    });

                    // Store result and jump to end
                    ctx.append(ir::Instruction::Copy {
                        src: ir::Value::Variable(right_bool),
                        dst: result.clone(),
                    });
                    ctx.append(ir::Instruction::Jump {
                        target: end_label.clone(),
                    });

                    // False label: set result to 0
                    ctx.append(ir::Instruction::Label(false_label));
                    ctx.append(ir::Instruction::Copy {
                        src: ir::Value::Constant(0),
                        dst: result.clone(),
                    });

                    // End label
                    ctx.append(ir::Instruction::Label(end_label));

                    return ir::Value::Variable(result);
                }

                ast::BinaryOperator::LogicalOr => {
                    // For: left || right
                    // If left is true (non-zero), result is 1 without evaluating right
                    // If left is false (0), result is (right != 0)

                    let result = ctx.alloc_var();
                    let true_label = ctx.alloc_label("or_true");
                    let end_label = ctx.alloc_label("or_end");

                    // Evaluate left operand
                    let v1 = flatten_expr(*left, ctx);

                    // If left is true (non-zero), jump to true_label
                    ctx.append(ir::Instruction::JumpIfNotZero {
                        condition: v1,
                        target: true_label.clone(),
                    });

                    // Left is false, evaluate right operand
                    let v2 = flatten_expr(*right, ctx);

                    // Convert right to boolean (0 or 1)
                    let right_bool = ctx.alloc_var();
                    ctx.append(ir::Instruction::Binary {
                        op: ir::BinaryOperator::NotEqual,
                        src1: v2,
                        src2: ir::Value::Constant(0),
                        dst: right_bool.clone(),
                    });

                    // Store result and jump to end
                    ctx.append(ir::Instruction::Copy {
                        src: ir::Value::Variable(right_bool),
                        dst: result.clone(),
                    });
                    ctx.append(ir::Instruction::Jump {
                        target: end_label.clone(),
                    });

                    // True label: set result to 1
                    ctx.append(ir::Instruction::Label(true_label));
                    ctx.append(ir::Instruction::Copy {
                        src: ir::Value::Constant(1),
                        dst: result.clone(),
                    });

                    // End label
                    ctx.append(ir::Instruction::Label(end_label));

                    return ir::Value::Variable(result);
                }

                _ => {
                    let v1 = flatten_expr(*left, ctx);
                    let v2 = flatten_expr(*right, ctx);
                    let dst = ctx.alloc_var();

                    ctx.append(ir::Instruction::Binary {
                        op: Context::convert_binary_op(&op),
                        src1: v1,
                        src2: v2,
                        dst: dst.clone(),
                    });

                    return ir::Value::Variable(dst);
                }
            }
        }

        ast::Expr::Variable(name) => ir::Value::Variable(name),

        ast::Expr::Conditional(condition, then_expr, else_expr) => {
            let result = ctx.alloc_var();
            let else_label = ctx.alloc_label("cond_else");
            let end_label = ctx.alloc_label("cond_end");

            let cond_val = flatten_expr(*condition, ctx);

            ctx.append(ir::Instruction::JumpIfZero {
                condition: cond_val,
                target: else_label.clone(),
            });

            let v1 = flatten_expr(*then_expr, ctx);
            ctx.append(ir::Instruction::Copy {
                src: v1,
                dst: result.clone(),
            });
            ctx.append(ir::Instruction::Jump {
                target: end_label.clone(),
            });

            ctx.append(ir::Instruction::Label(else_label));

            let v2 = flatten_expr(*else_expr, ctx);
            ctx.append(ir::Instruction::Copy {
                src: v2,
                dst: result.clone(),
            });

            ctx.append(ir::Instruction::Label(end_label));

            return ir::Value::Variable(result);
        }

        ast::Expr::Assignment(left, right) => {
            let dst = match *left {
                ast::Expr::Variable(name) => name,
                _ => unreachable!(),
            };

            let val = flatten_expr(*right, ctx);

            ctx.append(ir::Instruction::Copy {
                src: val,
                dst: dst.clone(),
            });

            return ir::Value::Variable(dst);
        }

        ast::Expr::CompoundAssignment(left, op, right) => {
            let var = match *left {
                ast::Expr::Variable(name) => name,
                _ => unreachable!(),
            };

            let rhs_val = flatten_expr(*right, ctx);

            let tmp = ctx.alloc_var();

            ctx.append(ir::Instruction::Binary {
                op: Context::convert_binary_op(&op),
                src1: ir::Value::Variable(var.clone()),
                src2: rhs_val,
                dst: tmp.clone(),
            });

            ctx.append(ir::Instruction::Copy {
                src: ir::Value::Variable(tmp.clone()),
                dst: var.clone(),
            });

            ir::Value::Variable(tmp)
        }
    }
}
