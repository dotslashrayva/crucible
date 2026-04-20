use crate::frontend::ast::*;

pub fn resolve(program: &mut Program) -> Result<(), String> {
    let mut counter: usize = 0;
    label_block(&mut program.function.body, &mut counter, None)
}

fn fresh_label(counter: &mut usize) -> String {
    let label = format!("loop.{}", *counter);
    *counter += 1;
    return label;
}

fn label_block(
    block: &mut Block,
    counter: &mut usize,
    current_loop: Option<&str>,
) -> Result<(), String> {
    for item in &mut block.items {
        if let BlockItem::Statement(stmt) = item {
            label_stmt(stmt, counter, current_loop)?;
        }
    }

    return Ok(());
}

fn label_stmt(
    stmt: &mut Statement,
    counter: &mut usize,
    current_loop: Option<&str>,
) -> Result<(), String> {
    match stmt {
        Statement::Break(label) => match current_loop {
            Some(l) => {
                *label = l.to_string();
                return Ok(());
            }

            None => Err("'break' statement outside of loop".to_string()),
        },

        Statement::Continue(label) => match current_loop {
            Some(l) => {
                *label = l.to_string();
                return Ok(());
            }

            None => Err("'continue' statement outside of loop".to_string()),
        },

        Statement::While(_, body, label) => {
            let new_label = fresh_label(counter);
            label_stmt(body, counter, Some(&new_label))?;
            *label = new_label;
            return Ok(());
        }

        Statement::DoWhile(body, _, label) => {
            let new_label = fresh_label(counter);
            label_stmt(body, counter, Some(&new_label))?;
            *label = new_label;
            return Ok(());
        }

        Statement::For(_, _, _, body, label) => {
            let new_label = fresh_label(counter);
            label_stmt(body, counter, Some(&new_label))?;
            *label = new_label;
            return Ok(());
        }

        Statement::If(_, then_s, else_s) => {
            label_stmt(then_s, counter, current_loop)?;

            if let Some(e) = else_s {
                label_stmt(e, counter, current_loop)?;
            }

            return Ok(());
        }

        Statement::Compound(block) => label_block(block, counter, current_loop),

        Statement::Labeled(_, inner) => label_stmt(inner, counter, current_loop),

        Statement::Return(_) | Statement::Expression(_) | Statement::Goto(_) | Statement::Null => {
            return Ok(());
        }
    }
}
