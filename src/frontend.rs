mod ast;
pub mod ir;
mod irgen;
mod lexer;
mod parser;
mod semantic;
mod token;

use crate::Stage;
use irgen::flatten;
use lexer::lex;
use parser::parse;
use semantic::analyze;

pub fn compile(source: String, stage: Stage) -> Result<Option<ir::Program>, String> {
    // Invoke Lexer
    let tokens = match lex(&source) {
        Ok(tokens) => tokens,
        Err(e) => return Err(format!("Lexical error: {}", e).into()),
    };
    if stage == Stage::Lex {
        dbg!(tokens);
        println!("Lexer OK!");
        return Ok(None);
    }

    // Invoke Parser
    let mut ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("Syntax error: {}", e).into()),
    };
    if stage == Stage::Parse {
        dbg!(ast);
        println!("Parser OK!");
        return Ok(None);
    }

    // Semantic Analysis
    if let Err(e) = analyze(&mut ast) {
        return Err(format!("Semantic error: {}", e).into());
    }
    if stage == Stage::Validate {
        dbg!(ast);
        println!("Validation OK!");
        return Ok(None);
    }

    // IR Generation
    let ir = flatten(ast);
    if stage == Stage::Ir {
        dbg!(ir);
        println!("IR OK!");
        return Ok(None);
    }

    return Ok(Some(ir));
}
