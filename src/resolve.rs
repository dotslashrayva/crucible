use crate::ast::*;
use std::collections::HashMap;

// Semantic Analysis Pass
// Context struct carries all the state we need during the walk
struct Context {
    scopes: Vec<HashMap<String, MapEntry>>,
    labels: HashMap<String, String>,
    var_counter: usize,
    label_counter: usize,
}

// Newtype pattern
#[derive(Clone)]
struct MapEntry {
    unique_name: String,
}

impl Context {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            labels: HashMap::new(),
            var_counter: 0,
            label_counter: 0,
        }
    }

    // Generate a unique variable name
    fn make_unique_var(&mut self, name: &str) -> String {
        let unique = format!("{}.{}", name, self.var_counter);
        self.var_counter += 1;
        unique
    }

    // Generate a unique loop label
    fn make_loop_label(&mut self) -> String {
        let label = format!("loop.{}", self.label_counter);
        self.label_counter += 1;
        label
    }

    // Generate a unique label for labeled statements
    fn make_unique_label(&mut self, name: &str) -> String {
        let label = format!("label.{}.{}", name, self.label_counter);
        self.label_counter += 1;
        label
    }

    // Scope Management
    // Push a new, empty scope onto the stack.
    fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    // Pop the innermost scope off the stack, instantly discarding its variables.
    fn exit_scope(&mut self) {
        self.scopes.pop().expect("Tried to pop the global scope!");
    }

    // Check if a variable is declared in the current (innermost) scope.
    // Useful for catching duplicate declarations in current scope.
    fn is_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains_key(name)
    }

    // Insert a new variable into the *current* (innermost) scope.
    fn insert_var(&mut self, original_name: String, unique_name: String) {
        let current_scope = self.scopes.last_mut().unwrap();
        current_scope.insert(original_name, MapEntry { unique_name });
    }

    // Look up a variable by searching backwards from the innermost scope
    // to the outermost scope. This handles shadowing
    fn lookup_var(&self, name: &str) -> Option<String> {
        // walk the vector backwards (stack)
        for scope in self.scopes.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry.unique_name.clone());
            }
        }

        // Not found in any scope
        return None;
    }
}

// Main (public) resolve function
// Runs semantic analysis on the whole program (in place).
pub fn resolve(program: &mut Program) -> Result<(), String> {
    let mut ctx = Context::new();
    resolve_function(&mut program.function, &mut ctx)
}

// Functions & Blocks
fn resolve_function(func: &mut Function, ctx: &mut Context) -> Result<(), String> {
    collect_labels(&func.body, ctx)?;
    resolve_block(&mut func.body, ctx, &None)
}

fn resolve_block(
    block: &mut Block,
    ctx: &mut Context,
    current_label: &Option<String>,
) -> Result<(), String> {
    for item in &mut block.items {
        resolve_block_item(item, ctx, current_label)?;
    }

    return Ok(());
}

fn resolve_block_item(
    item: &mut BlockItem,
    ctx: &mut Context,
    current_label: &Option<String>,
) -> Result<(), String> {
    match item {
        BlockItem::Declaration(decl) => resolve_declaration(decl, ctx),
        BlockItem::Statement(stmt) => resolve_statement(stmt, ctx, current_label),
    }
}

fn resolve_declaration(decl: &mut Declaration, ctx: &mut Context) -> Result<(), String> {
    let original_name = decl.name.clone();

    // Reject duplicate declarations inside the same block.
    // We just check the top of the stack.
    if ctx.is_in_current_scope(&original_name) {
        return Err(format!(
            "Duplicate variable declaration: '{}'",
            original_name
        ));
    }

    // Generate a unique name and register it in the current scope.
    let unique_name = ctx.make_unique_var(&original_name);
    ctx.insert_var(original_name, unique_name.clone());

    // Resolve the initializer expr, if there is one.
    if let Some(init) = &mut decl.init {
        resolve_exp(init, ctx)?;
    }

    // Rename the declaration.
    decl.name = unique_name;

    return Ok(());
}

fn resolve_statement(
    stmt: &mut Statement,
    ctx: &mut Context,
    current_label: &Option<String>,
) -> Result<(), String> {
    match stmt {
        Statement::Return(expr) => resolve_exp(expr, ctx),
        Statement::Expression(expr) => resolve_exp(expr, ctx),
        Statement::Null => Ok(()),

        // Break / Continue: attach the current loop's label
        // If we're not inside any loop, that's an error.
        Statement::Break(label) => match current_label {
            Some(l) => {
                *label = l.clone();
                Ok(())
            }
            None => Err("'break' statement outside of loop".to_string()),
        },

        Statement::Continue(label) => match current_label {
            Some(l) => {
                *label = l.clone();
                Ok(())
            }
            None => Err("'continue' statement outside of loop".to_string()),
        },

        // If: resolve condition, then both branches
        Statement::If(condition, then_stmt, else_stmt) => {
            resolve_exp(condition, ctx)?;
            resolve_statement(then_stmt, ctx, current_label)?;

            if let Some(else_branch) = else_stmt {
                resolve_statement(else_branch, ctx, current_label)?;
            }

            Ok(())
        }

        // Compound: enter a new scope
        Statement::Compound(block) => {
            ctx.enter_scope();
            resolve_block(block, ctx, current_label)?;
            ctx.exit_scope();
            Ok(())
        }

        // While loop: generate a label, resolve cond + body
        Statement::While(condition, body, label) => {
            let new_label = ctx.make_loop_label();

            resolve_exp(condition, ctx)?;
            resolve_statement(body, ctx, &Some(new_label.clone()))?;

            *label = new_label;
            Ok(())
        }

        // Do-While: same idea, body comes before condition
        Statement::DoWhile(body, condition, label) => {
            let new_label = ctx.make_loop_label();

            resolve_statement(body, ctx, &Some(new_label.clone()))?;
            resolve_exp(condition, ctx)?;

            *label = new_label;
            Ok(())
        }

        // For loop: new scope (for the loop variable) + new label
        Statement::For(init, condition, post, body, label) => {
            let new_label = ctx.make_loop_label();

            // The for-loop header introduces its own scope that
            // they doesn't leak outside the loop.
            ctx.enter_scope();

            resolve_for_init(init, ctx)?;
            resolve_optional_exp(condition, ctx)?;
            resolve_optional_exp(post, ctx)?;
            resolve_statement(body, ctx, &Some(new_label.clone()))?;

            ctx.exit_scope();

            *label = new_label;
            Ok(())
        }

        // Goto <label>
        Statement::Goto(label) => match ctx.labels.get(label) {
            Some(unique) => {
                *label = unique.clone();
                Ok(())
            }
            None => Err(format!("Undefined label: '{}'", label)),
        },

        // <label>
        Statement::Labeled(name, inner) => {
            let unique = ctx.labels.get(name).unwrap().clone();
            *name = unique;
            resolve_statement(inner, ctx, current_label)
        }
    }
}

fn resolve_for_init(init: &mut ForInit, ctx: &mut Context) -> Result<(), String> {
    match init {
        ForInit::InitDecl(decl) => resolve_declaration(decl, ctx),
        ForInit::InitExpr(expr) => resolve_optional_exp(expr, ctx),
    }
}

fn resolve_optional_exp(expr: &mut Option<Expr>, ctx: &mut Context) -> Result<(), String> {
    if let Some(e) = expr {
        resolve_exp(e, ctx)?;
    }
    Ok(())
}

fn resolve_exp(expr: &mut Expr, ctx: &mut Context) -> Result<(), String> {
    match expr {
        // Constants don't reference any variables, nothing to do.
        Expr::Constant(_) => Ok(()),

        // Variables: look up the source name and replace it with the
        // unique name we assigned during declaration.
        Expr::Variable(name) => match ctx.lookup_var(name) {
            Some(unique_name) => {
                *name = unique_name;
                Ok(())
            }
            None => Err(format!("Undeclared variable: '{}'", name)),
        },

        // Assignments: the left-hand side must be a variable (lvalue check),
        // then resolve both sides.
        Expr::Assignment(left, right) => {
            if !matches!(left.as_ref(), Expr::Variable(_)) {
                return Err("Invalid lvalue in assignment".to_string());
            }

            resolve_exp(left, ctx)?;
            resolve_exp(right, ctx)
        }

        Expr::CompoundAssignment(left, _op, right) => {
            if !matches!(left.as_ref(), Expr::Variable(_)) {
                return Err("Invalid lvalue in compound assignment".to_string());
            }

            resolve_exp(left, ctx)?;
            resolve_exp(right, ctx)
        }

        // Unary / Binary / Conditional: recurse into sub-expressions.
        // Prefix ++/--: operand must be an lvalue
        Expr::Unary(op, inner) => {
            if matches!(
                op,
                UnaryOperator::PrefixIncrement | UnaryOperator::PrefixDecrement
            ) {
                if !matches!(inner.as_ref(), Expr::Variable(_)) {
                    return Err("Invalid lvalue in prefix operation".to_string());
                }
            }

            resolve_exp(inner, ctx)
        }

        Expr::Binary(_op, left, right) => {
            resolve_exp(left, ctx)?;
            resolve_exp(right, ctx)
        }

        Expr::Conditional(condition, then_expr, else_expr) => {
            resolve_exp(condition, ctx)?;
            resolve_exp(then_expr, ctx)?;
            resolve_exp(else_expr, ctx)
        }

        // Postfix ++/--: operand must be an lvalue
        Expr::PostfixIncrement(inner) | Expr::PostfixDecrement(inner) => {
            if !matches!(inner.as_ref(), Expr::Variable(_)) {
                return Err("Invalid lvalue in postfix operation".to_string());
            }
            resolve_exp(inner, ctx)
        }
    }
}

fn collect_labels(block: &Block, ctx: &mut Context) -> Result<(), String> {
    for item in &block.items {
        collect_labels_in_block_item(item, ctx)?;
    }
    Ok(())
}

fn collect_labels_in_block_item(item: &BlockItem, ctx: &mut Context) -> Result<(), String> {
    match item {
        BlockItem::Statement(stmt) => collect_labels_in_stmt(stmt, ctx),
        BlockItem::Declaration(_) => Ok(()),
    }
}

fn collect_labels_in_stmt(stmt: &Statement, ctx: &mut Context) -> Result<(), String> {
    match stmt {
        Statement::Labeled(name, inner) => {
            if ctx.labels.contains_key(name) {
                return Err(format!("Duplicate label: '{}'", name));
            }
            let unique = ctx.make_unique_label(name);
            ctx.labels.insert(name.clone(), unique);
            collect_labels_in_stmt(inner, ctx)
        }

        // Recurse into anything that can contain statements
        Statement::If(_, then_s, else_s) => {
            collect_labels_in_stmt(then_s, ctx)?;
            if let Some(e) = else_s {
                collect_labels_in_stmt(e, ctx)?;
            }
            Ok(())
        }

        Statement::Compound(block) => collect_labels(block, ctx),
        Statement::While(_, body, _) => collect_labels_in_stmt(body, ctx),
        Statement::DoWhile(body, _, _) => collect_labels_in_stmt(body, ctx),
        Statement::For(_, _, _, body, _) => collect_labels_in_stmt(body, ctx),
        _ => Ok(()),
    }
}
