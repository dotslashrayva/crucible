use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

mod asm;
mod ast;
mod codegen;
mod emit;
mod ir;
mod irgen;
mod lexer;
mod parser;
mod resolve;
mod token;

use codegen::generate;
use emit::emit;
use irgen::flatten;
use lexer::lex;
use parser::parse;
use resolve::resolve;

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: crucible <flag> <source.c>");
        eprintln!("Flags: [--lex OR --parse OR --codegen]");
        return Err("no arguments provided".into());
    }

    let mut stop_after_lex: bool = false;
    let mut stop_after_parse: bool = false;
    let mut stop_after_validate: bool = false;
    let mut stop_after_ir: bool = false;
    let mut stop_after_codegen: bool = false;
    let mut stop_after_emit: bool = false;

    let mut input_path: String = String::new();

    for arg in &args {
        match arg.as_str() {
            "--lex" => stop_after_lex = true,
            "--parse" => stop_after_parse = true,
            "--validate" => stop_after_validate = true,
            "--tacky" | "--ir" => stop_after_ir = true,
            "--codegen" => stop_after_codegen = true,
            "-S" | "--emit" => stop_after_emit = true,
            _ => input_path = arg.to_string(),
        }
    }

    let input = Path::new(&input_path);
    let output = input.with_extension("i");

    let prep_status = Command::new("clang")
        .arg("-E")
        .arg("-P")
        .arg(&input)
        .arg("-o")
        .arg(&output)
        .status()
        .expect("failed to run clang");

    if !prep_status.success() {
        return Err("clang failed to preprocess".into());
    }

    let source = fs::read_to_string(&output)?;
    fs::remove_file(&output)?;

    let tokens = match lex(&source) {
        Ok(tokens) => tokens,
        Err(e) => return Err(format!("Lexical error: {}", e).into()),
    };

    if stop_after_lex {
        dbg!(tokens);
        println!("Lexer OK!");
        return Ok(());
    }

    let ast = match parse(tokens) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("Syntax error: {}", e).into()),
    };

    if stop_after_parse {
        dbg!(ast);
        println!("Parser OK!");
        return Ok(());
    }

    let ast = match resolve(ast) {
        Ok(ast) => ast,
        Err(e) => return Err(format!("Semantic error: {}", e).into()),
    };

    if stop_after_validate {
        dbg!(ast);
        println!("Validation OK!");
        return Ok(());
    }

    let ir = flatten(ast);

    if stop_after_ir {
        dbg!(ir);
        println!("IR OK!");
        return Ok(());
    }

    let assembly = generate(ir);

    if stop_after_codegen {
        dbg!(assembly);
        println!("Code Generation OK!");
        return Ok(());
    }

    let assembly_code = emit(assembly);

    if stop_after_emit {
        println!("{}", assembly_code);
        println!("Code Emission OK!");
        return Ok(());
    }

    // Save the Code and Invoke Assembler
    let asm_file = input.with_extension("s");
    let exec_file = input.with_extension("");
    fs::write(&asm_file, assembly_code)?;

    let assembler_status = Command::new("clang")
        .arg("-target")
        .arg("x86_64-apple-darwin")
        .arg(&asm_file)
        .arg("-o")
        .arg(&exec_file)
        .status()
        .expect("failed to run clang");

    if !assembler_status.success() {
        return Err(format!("clang failed to assemble and link").into());
    }

    fs::remove_file(&asm_file)?;

    return Ok(());
}
