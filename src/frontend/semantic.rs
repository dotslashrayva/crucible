mod gotos;
mod loops;
mod variable;

use crate::frontend::ast::Program;

pub fn analyze(program: &mut Program) -> Result<(), String> {
    variable::resolve(program)?;
    gotos::resolve(program)?;
    loops::resolve(program)?;
    return Ok(());
}
