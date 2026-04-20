use crate::frontend::ast::*;
use std::collections::HashMap;

struct ScopeStack {
    scopes: Vec<HashMap<String, String>>,
    counter: usize,
}

impl ScopeStack {
    fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            counter: 0,
        }
    }

    fn enter(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn exit(&mut self) {
        self.scopes.pop().expect("tried to pop global scope");
    }

    fn declared_here(&self, name: &str) -> bool {
        self.scopes.last().unwrap().contains_key(name)
    }

    fn declare(&mut self, name: &str) -> String {
        let unique = format!("{}.{}", name, self.counter);

        self.counter += 1;
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), unique.clone());

        return unique;
    }

    fn lookup(&self, name: &str) -> Option<String> {
        for scope in self.scopes.iter().rev() {
            if let Some(info) = scope.get(name) {
                return Some(info.clone());
            }
        }
        return None;
    }
}

pub fn resolve(program: &mut Program) -> Result<(), String> {
    let mut scopes = ScopeStack::new();
    return resolve_block(&mut program.function.body, &mut scopes);
}

fn resolve_block(block: &mut Block, scopes: &mut ScopeStack) -> Result<(), String> {
    for item in &mut block.items {
        match item {
            BlockItem::Declaration(decl) => resolve_decl(decl, scopes)?,
            BlockItem::Statement(stmt) => resolve_stmt(stmt, scopes)?,
        }
    }
    return Ok(());
}

fn resolve_decl(decl: &mut Declaration, scopes: &mut ScopeStack) -> Result<(), String> {
    if scopes.declared_here(&decl.name) {
        return Err(format!("duplicate variable declaration: '{}'", decl.name));
    }

    let unique = scopes.declare(&decl.name);

    if let Some(init) = &mut decl.init {
        resolve_expr(init, scopes)?;
    }

    decl.name = unique;
    return Ok(());
}

fn resolve_stmt(stmt: &mut Statement, scopes: &mut ScopeStack) -> Result<(), String> {
    match stmt {
        Statement::Return(e) | Statement::Expression(e) => resolve_expr(e, scopes),

        Statement::Null => Ok(()),

        Statement::Break(_) | Statement::Continue(_) | Statement::Goto(_) => Ok(()),

        Statement::If(cond, then_s, else_s) => {
            resolve_expr(cond, scopes)?;
            resolve_stmt(then_s, scopes)?;

            if let Some(e) = else_s {
                resolve_stmt(e, scopes)?;
            }

            return Ok(());
        }

        Statement::Compound(block) => {
            scopes.enter();
            let result = resolve_block(block, scopes);
            scopes.exit();
            return result;
        }

        Statement::While(cond, body, _) => {
            resolve_expr(cond, scopes)?;
            resolve_stmt(body, scopes)
        }

        Statement::DoWhile(body, cond, _) => {
            resolve_stmt(body, scopes)?;
            resolve_expr(cond, scopes)
        }

        Statement::For(init, cond, post, body, _) => {
            scopes.enter();
            resolve_for_init(init, scopes)?;

            if let Some(c) = cond {
                resolve_expr(c, scopes)?;
            }

            if let Some(p) = post {
                resolve_expr(p, scopes)?;
            }

            resolve_stmt(body, scopes)?;
            scopes.exit();

            return Ok(());
        }

        Statement::Labeled(_, inner) => resolve_stmt(inner, scopes),
    }
}

fn resolve_for_init(init: &mut ForInit, scopes: &mut ScopeStack) -> Result<(), String> {
    match init {
        ForInit::InitDecl(decl) => resolve_decl(decl, scopes),
        ForInit::InitExpr(Some(e)) => resolve_expr(e, scopes),
        ForInit::InitExpr(None) => Ok(()),
    }
}

fn resolve_expr(expr: &mut Expr, scopes: &mut ScopeStack) -> Result<(), String> {
    match expr {
        Expr::Constant(_) => Ok(()),

        Expr::Variable(name) => match scopes.lookup(name) {
            Some(unique) => {
                *name = unique;
                return Ok(());
            }

            None => Err(format!("undeclared variable: '{}'", name)),
        },

        Expr::Assignment(left, right) => {
            require_lvalue(left.as_ref(), "assignment")?;
            resolve_expr(left, scopes)?;
            resolve_expr(right, scopes)
        }

        Expr::CompoundAssignment(left, _op, right) => {
            require_lvalue(left.as_ref(), "compound assignment")?;
            resolve_expr(left, scopes)?;
            resolve_expr(right, scopes)
        }

        Expr::Unary(op, inner) => {
            if matches!(
                op,
                UnaryOperator::PrefixIncrement | UnaryOperator::PrefixDecrement
            ) {
                require_lvalue(inner.as_ref(), "prefix ++/--")?;
            }

            resolve_expr(inner, scopes)
        }

        Expr::PostfixIncrement(inner) | Expr::PostfixDecrement(inner) => {
            require_lvalue(inner.as_ref(), "postfix ++/--")?;
            resolve_expr(inner, scopes)
        }

        Expr::Binary(_, l, r) => {
            resolve_expr(l, scopes)?;
            resolve_expr(r, scopes)
        }

        Expr::Conditional(c, t, e) => {
            resolve_expr(c, scopes)?;
            resolve_expr(t, scopes)?;
            resolve_expr(e, scopes)
        }
    }
}

fn require_lvalue(expr: &Expr, context: &str) -> Result<(), String> {
    if matches!(expr, Expr::Variable(_)) {
        return Ok(());
    } else {
        Err(format!("invalid lvalue in {}", context))
    }
}
