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

fn resolve_optional_exp(expr: Option<Expr>, ctx: &mut Context) -> Result<Option<Expr>, String> {
    match expr {
        Some(e) => Ok(Some(resolve_exp(e, ctx)?)),
        None => Ok(None),
    }
}

fn resolve_for_init(init: ForInit, ctx: &mut Context) -> Result<ForInit, String> {
    match init {
        ForInit::InitDecl(decl) => {
            let resolved = resolve_declaration(decl, ctx)?;
            Ok(ForInit::InitDecl(resolved))
        }
        ForInit::InitExp(expr) => {
            let resolved = resolve_optional_exp(expr, ctx)?;
            Ok(ForInit::InitExp(resolved))
        }
    }
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

        // Break and Continue have no subexpressions to resolve
        Statement::Break(label) => Ok(Statement::Break(label)),
        Statement::Continue(label) => Ok(Statement::Continue(label)),

        Statement::While(condition, body, label) => {
            let resolved_cond = resolve_exp(condition, ctx)?;
            let resolved_body = resolve_statement(*body, ctx)?;
            Ok(Statement::While(
                resolved_cond,
                Box::new(resolved_body),
                label,
            ))
        }

        Statement::DoWhile(body, condition, label) => {
            let resolved_body = resolve_statement(*body, ctx)?;
            let resolved_cond = resolve_exp(condition, ctx)?;
            Ok(Statement::DoWhile(
                Box::new(resolved_body),
                resolved_cond,
                label,
            ))
        }

        Statement::For(init, condition, post, body, label) => {
            // For loop header introduces a new scope
            let saved_map = ctx.variable_map.clone();
            ctx.variable_map = ctx.copy_variable_map();

            let resolved_init = resolve_for_init(init, ctx)?;
            let resolved_cond = resolve_optional_exp(condition, ctx)?;
            let resolved_post = resolve_optional_exp(post, ctx)?;
            let resolved_body = resolve_statement(*body, ctx)?;

            // Restore the outer scope's variable map
            ctx.variable_map = saved_map;

            Ok(Statement::For(
                resolved_init,
                resolved_cond,
                resolved_post,
                Box::new(resolved_body),
                label,
            ))
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

// Loop Labeling

struct LabelContext {
    counter: usize,
}

impl LabelContext {
    fn new() -> Self {
        Self { counter: 0 }
    }

    fn make_label(&mut self) -> String {
        let label = format!("loop.{}", self.counter);
        self.counter += 1;
        label
    }
}

pub fn label_loops(program: Program) -> Result<Program, String> {
    let mut ctx = LabelContext::new();
    let function = label_function(program.function, &mut ctx)?;
    Ok(Program { function })
}

fn label_function(func: Function, ctx: &mut LabelContext) -> Result<Function, String> {
    let labeled_body = label_block(func.body, ctx, &None)?;
    Ok(Function {
        name: func.name,
        body: labeled_body,
    })
}

fn label_block(
    block: Block,
    ctx: &mut LabelContext,
    current_label: &Option<String>,
) -> Result<Block, String> {
    let mut labeled_items = Vec::new();

    for item in block.items {
        let labeled = label_block_item(item, ctx, current_label)?;
        labeled_items.push(labeled);
    }

    Ok(Block {
        items: labeled_items,
    })
}

fn label_block_item(
    item: BlockItem,
    ctx: &mut LabelContext,
    current_label: &Option<String>,
) -> Result<BlockItem, String> {
    match item {
        BlockItem::Declare(decl) => Ok(BlockItem::Declare(decl)),
        BlockItem::State(stmt) => {
            let labeled = label_statement(stmt, ctx, current_label)?;
            Ok(BlockItem::State(labeled))
        }
    }
}

fn label_statement(
    stmt: Statement,
    ctx: &mut LabelContext,
    current_label: &Option<String>,
) -> Result<Statement, String> {
    match stmt {
        Statement::Break(_) => match current_label {
            Some(label) => Ok(Statement::Break(label.clone())),
            None => Err("'break' statement outside of loop".to_string()),
        },

        Statement::Continue(_) => match current_label {
            Some(label) => Ok(Statement::Continue(label.clone())),
            None => Err("'continue' statement outside of loop".to_string()),
        },

        Statement::While(condition, body, _) => {
            let new_label = ctx.make_label();
            let labeled_body = label_statement(*body, ctx, &Some(new_label.clone()))?;
            Ok(Statement::While(
                condition,
                Box::new(labeled_body),
                new_label,
            ))
        }

        Statement::DoWhile(body, condition, _) => {
            let new_label = ctx.make_label();
            let labeled_body = label_statement(*body, ctx, &Some(new_label.clone()))?;
            Ok(Statement::DoWhile(
                Box::new(labeled_body),
                condition,
                new_label,
            ))
        }

        Statement::For(init, condition, post, body, _) => {
            let new_label = ctx.make_label();
            let labeled_body = label_statement(*body, ctx, &Some(new_label.clone()))?;
            Ok(Statement::For(
                init,
                condition,
                post,
                Box::new(labeled_body),
                new_label,
            ))
        }

        Statement::If(condition, then_stmt, else_stmt) => {
            let labeled_then = label_statement(*then_stmt, ctx, current_label)?;
            let labeled_else = match else_stmt {
                Some(stmt) => Some(Box::new(label_statement(*stmt, ctx, current_label)?)),
                None => None,
            };
            Ok(Statement::If(
                condition,
                Box::new(labeled_then),
                labeled_else,
            ))
        }

        Statement::Compound(block) => {
            let labeled_block = label_block(block, ctx, current_label)?;
            Ok(Statement::Compound(labeled_block))
        }

        // These don't contain sub-statements, pass through unchanged
        Statement::Return(_) | Statement::Expression(_) | Statement::Null => Ok(stmt),
    }
}
