use std::collections::HashMap;

use crate::ast::*;

struct Context {
    variable_map: HashMap<String, String>,
    counter: usize,
}

impl Context {
    fn new() -> Self {
        Self {
            variable_map: HashMap::new(),
            counter: 0,
        }
    }

    // We just rename the variable
    fn make_temporary(&mut self, name: &str) -> String {
        let unique = format!("{}.{}", name, self.counter);
        self.counter += 1;
        return unique;
    }
}

// Main resolve function
pub fn resolve(program: Program) -> Result<Program, String> {
    let function = resolve_function(program.function)?;
    return Ok(Program { function });
}

fn resolve_function(func: Function) -> Result<Function, String> {
    let mut ctx = Context::new();
    let mut resolved_body = Vec::new();

    for block in func.body {
        let resolved = resolve_block_item(block, &mut ctx)?;
        resolved_body.push(resolved);
    }

    return Ok(Function {
        name: func.name,
        body: resolved_body,
    });
}

fn resolve_block_item(block: Block, ctx: &mut Context) -> Result<Block, String> {
    match block {
        Block::Declare(decl) => {
            let resolved = resolve_declaration(decl, ctx)?;
            Ok(Block::Declare(resolved))
        }
        Block::State(stmt) => {
            let resolved = resolve_statement(stmt, ctx)?;
            Ok(Block::State(resolved))
        }
    }
}

fn resolve_declaration(decl: Declaration, ctx: &mut Context) -> Result<Declaration, String> {
    if ctx.variable_map.contains_key(&decl.name) {
        return Err(format!("Duplicate variable declaration: '{}'", decl.name));
    }

    let unique_name = ctx.make_temporary(&decl.name);
    ctx.variable_map.insert(decl.name, unique_name.clone());

    let init = match decl.init {
        Some(expr) => Some(resolve_exp(expr, ctx)?),
        None => None,
    };

    return Ok(Declaration {
        name: unique_name,
        init,
    });
}

fn resolve_statement(stmt: Statement, ctx: &mut Context) -> Result<Statement, String> {
    match stmt {
        Statement::Return(expr) => {
            let resolved = resolve_exp(expr, ctx)?;
            Ok(Statement::Return(resolved))
        }
        Statement::Expression(expr) => {
            let resolved = resolve_exp(expr, ctx)?;
            Ok(Statement::Expression(resolved))
        }
        Statement::Null => Ok(Statement::Null),
    }
}

fn resolve_exp(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    match expr {
        Expr::Constant(val) => return Ok(Expr::Constant(val)),

        Expr::Variable(name) => match ctx.variable_map.get(&name) {
            Some(unique_name) => return Ok(Expr::Variable(unique_name.clone())),
            None => return Err(format!("Undeclared variable: '{}'", name)),
        },

        Expr::Assignment(left, right) => {
            if !matches!(*left, Expr::Variable(_)) {
                return Err("Invalid lvalue in assignment".to_string());
            }

            let resolved_left = resolve_exp(*left, ctx)?;
            let resolved_right = resolve_exp(*right, ctx)?;
            return Ok(Expr::Assignment(
                Box::new(resolved_left),
                Box::new(resolved_right),
            ));
        }

        Expr::Unary(op, inner) => {
            let resolved = resolve_exp(*inner, ctx)?;
            return Ok(Expr::Unary(op, Box::new(resolved)));
        }

        Expr::Binary(op, left, right) => {
            let resolved_left = resolve_exp(*left, ctx)?;
            let resolved_right = resolve_exp(*right, ctx)?;
            return Ok(Expr::Binary(
                op,
                Box::new(resolved_left),
                Box::new(resolved_right),
            ));
        }
    }
}
