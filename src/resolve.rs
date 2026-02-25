use std::collections::HashMap;

use crate::ast::*;

#[derive(Clone)]
struct MapEntry {
    unique_name: String,
    from_current_block: bool,
}

struct Context {
    variable_map: HashMap<String, MapEntry>,
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

    // Create a copy of the variable map
    fn copy_variable_map(&self) -> HashMap<String, MapEntry> {
        let mut new_map = HashMap::new();

        for (name, entry) in &self.variable_map {
            new_map.insert(
                name.clone(),
                MapEntry {
                    unique_name: entry.unique_name.clone(),
                    from_current_block: false,
                },
            );
        }

        return new_map;
    }
}

// Main resolve function
pub fn resolve(program: Program) -> Result<Program, String> {
    let function = resolve_function(program.function)?;
    return Ok(Program { function });
}

fn resolve_function(func: Function) -> Result<Function, String> {
    let mut ctx = Context::new();
    let resolved_body = resolve_block(func.body, &mut ctx)?;

    return Ok(Function {
        name: func.name,
        body: resolved_body,
    });
}

fn resolve_block(block: Block, ctx: &mut Context) -> Result<Block, String> {
    let mut resolved_items = Vec::new();

    for item in block.items {
        let resolved = resolve_block_item(item, ctx)?;
        resolved_items.push(resolved);
    }

    return Ok(Block {
        items: resolved_items,
    });
}

fn resolve_block_item(item: BlockItem, ctx: &mut Context) -> Result<BlockItem, String> {
    match item {
        BlockItem::Declare(decl) => {
            let resolved = resolve_declaration(decl, ctx)?;
            Ok(BlockItem::Declare(resolved))
        }
        BlockItem::State(stmt) => {
            let resolved = resolve_statement(stmt, ctx)?;
            Ok(BlockItem::State(resolved))
        }
    }
}

fn resolve_declaration(decl: Declaration, ctx: &mut Context) -> Result<Declaration, String> {
    // Check for duplicate declaration in the CURRENT block
    if let Some(entry) = ctx.variable_map.get(&decl.name) {
        if entry.from_current_block {
            return Err(format!("Duplicate variable declaration: '{}'", decl.name));
        }
    }

    let unique_name = ctx.make_temporary(&decl.name);
    ctx.variable_map.insert(
        decl.name,
        MapEntry {
            unique_name: unique_name.clone(),
            from_current_block: true,
        },
    );

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

        Statement::If(condition, then_stmt, else_stmt) => {
            let resolved_cond = resolve_exp(condition, ctx)?;
            let resolved_then = resolve_statement(*then_stmt, ctx)?;
            let resolved_else = match else_stmt {
                Some(stmt) => Some(Box::new(resolve_statement(*stmt, ctx)?)),
                None => None,
            };
            Ok(Statement::If(
                resolved_cond,
                Box::new(resolved_then),
                resolved_else,
            ))
        }

        Statement::Compound(block) => {
            // Copy the variable map with from_current_block reset to false,
            // so inner declarations can shadow outer ones without error
            let saved_map = ctx.variable_map.clone();
            ctx.variable_map = ctx.copy_variable_map();

            let resolved_block = resolve_block(block, ctx)?;

            // Restore the outer scope's variable map
            ctx.variable_map = saved_map;

            Ok(Statement::Compound(resolved_block))
        }

        Statement::Null => Ok(Statement::Null),
    }
}

fn resolve_exp(expr: Expr, ctx: &mut Context) -> Result<Expr, String> {
    match expr {
        Expr::Constant(val) => return Ok(Expr::Constant(val)),

        Expr::Variable(name) => match ctx.variable_map.get(&name) {
            Some(entry) => return Ok(Expr::Variable(entry.unique_name.clone())),
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

        Expr::Conditional(condition, then_expr, else_expr) => {
            let resolved_cond = resolve_exp(*condition, ctx)?;
            let resolved_then = resolve_exp(*then_expr, ctx)?;
            let resolved_else = resolve_exp(*else_expr, ctx)?;
            return Ok(Expr::Conditional(
                Box::new(resolved_cond),
                Box::new(resolved_then),
                Box::new(resolved_else),
            ));
        }
    }
}
