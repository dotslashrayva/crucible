use crate::frontend::ast::*;
use std::collections::HashMap;

type LabelMap = HashMap<String, String>;

pub fn resolve(program: &mut Program) -> Result<(), String> {
    let mut labels: LabelMap = HashMap::new();
    let mut counter: usize = 0;

    collect_block(&program.function.body, &mut labels, &mut counter)?;
    rewrite_block(&mut program.function.body, &labels)
}

fn collect_block(block: &Block, labels: &mut LabelMap, counter: &mut usize) -> Result<(), String> {
    for item in &block.items {
        if let BlockItem::Statement(stmt) = item {
            collect_stmt(stmt, labels, counter)?;
        }
    }
    return Ok(());
}

fn collect_stmt(
    stmt: &Statement,
    labels: &mut LabelMap,
    counter: &mut usize,
) -> Result<(), String> {
    match stmt {
        Statement::Labeled(name, inner) => {
            if labels.contains_key(name) {
                return Err(format!("duplicate label: '{}'", name));
            }

            let unique = format!("label.{}.{}", name, *counter);
            *counter += 1;

            labels.insert(name.clone(), unique);
            collect_stmt(inner, labels, counter)
        }

        Statement::If(_, then_s, else_s) => {
            collect_stmt(then_s, labels, counter)?;

            if let Some(e) = else_s {
                collect_stmt(e, labels, counter)?;
            }

            return Ok(());
        }

        Statement::Compound(block) => collect_block(block, labels, counter),
        Statement::While(_, body, _) => collect_stmt(body, labels, counter),
        Statement::DoWhile(body, _, _) => collect_stmt(body, labels, counter),
        Statement::For(_, _, _, body, _) => collect_stmt(body, labels, counter),

        Statement::Return(_)
        | Statement::Expression(_)
        | Statement::Null
        | Statement::Break(_)
        | Statement::Continue(_)
        | Statement::Goto(_) => Ok(()),
    }
}

fn rewrite_block(block: &mut Block, labels: &LabelMap) -> Result<(), String> {
    for item in &mut block.items {
        if let BlockItem::Statement(stmt) = item {
            rewrite_stmt(stmt, labels)?;
        }
    }

    return Ok(());
}

fn rewrite_stmt(stmt: &mut Statement, labels: &LabelMap) -> Result<(), String> {
    match stmt {
        Statement::Labeled(name, inner) => {
            *name = labels.get(name).unwrap().clone();
            return rewrite_stmt(inner, labels);
        }

        Statement::Goto(target) => match labels.get(target) {
            Some(unique) => {
                *target = unique.clone();
                return Ok(());
            }

            None => Err(format!("undefined label: '{}'", target)),
        },

        Statement::If(_, then_s, else_s) => {
            rewrite_stmt(then_s, labels)?;

            if let Some(e) = else_s {
                rewrite_stmt(e, labels)?;
            }

            return Ok(());
        }

        Statement::Compound(block) => rewrite_block(block, labels),
        Statement::While(_, body, _) => rewrite_stmt(body, labels),
        Statement::DoWhile(body, _, _) => rewrite_stmt(body, labels),
        Statement::For(_, _, _, body, _) => rewrite_stmt(body, labels),

        Statement::Return(_)
        | Statement::Expression(_)
        | Statement::Null
        | Statement::Break(_)
        | Statement::Continue(_) => Ok(()),
    }
}
