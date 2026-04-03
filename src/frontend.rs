pub mod ast;
pub mod ir;
pub mod irgen;
pub mod lexer;
pub mod parser;
pub mod resolve;
pub mod token;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Stage {
    Lex,
    Parse,
    Validate,
    Ir,
    Codegen,
    Emit,
    Full,
}

pub fn run_pipeline(source: &str, stage: Stage) -> Result<Option<ir::Program>, String> {
    let tokens = lexer::lex(source).map_err(|e| format!("Lexical error: {}", e))?;
    if stage == Stage::Lex {
        dbg!(&tokens);
        println!("Lexer OK!");
        return Ok(None);
    }

    let mut ast = parser::parse(tokens).map_err(|e| format!("Syntax error: {}", e))?;
    if stage == Stage::Parse {
        dbg!(&ast);
        println!("Parser OK!");
        return Ok(None);
    }

    resolve::resolve(&mut ast).map_err(|e| format!("Semantic error: {}", e))?;
    if stage == Stage::Validate {
        dbg!(&ast);
        println!("Validation OK!");
        return Ok(None);
    }

    let ir = irgen::flatten(ast);
    if stage == Stage::Ir {
        dbg!(&ir);
        println!("IR OK!");
        return Ok(None);
    }

    Ok(Some(ir))
}
