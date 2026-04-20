use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command;

mod backend;
mod frontend;

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

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: crucible <flag> <source.c>");
        eprintln!("Flags: [--lex OR --parse OR --validate]");
        eprintln!("Flags: [--ir OR --codegen OR --emit]");
        return Err("no arguments provided".into());
    }

    let mut stage = Stage::Full;
    let mut input_path: String = String::new();

    for arg in &args {
        match arg.as_str() {
            "--lex" => stage = Stage::Lex,
            "--parse" => stage = Stage::Parse,
            "--validate" => stage = Stage::Validate,
            "--codegen" => stage = Stage::Codegen,

            "--tacky" | "--ir" => stage = Stage::Ir,
            "-S" | "--emit" => stage = Stage::Emit,

            "--version" | "-v" => {
                println!("crucible version 0.1.0");
                println!("target: x86_64-apple-darwin");
                return Ok(());
            }

            "--help" | "-h" => {
                println!("Usage: crucible <flag> <source.c>");
                println!("Optional Flags: [--lex OR --parse OR --validate]");
                println!("Optional Flags: [--ir OR --codegen OR --emit]");
                return Ok(());
            }

            flag if flag.starts_with('-') => return Err(format!("Unknown flag: {}", flag).into()),
            file => input_path = file.to_string(),
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

    let Some(ir) = frontend::compile(source, stage)? else {
        return Ok(()); // Stage ended early
    };

    let Some(assembly_code) = backend::compile(ir, stage) else {
        return Ok(()); // Stage ended early
    };

    // Write the assembly code
    let asm_file = input.with_extension("s");
    let exec_file = input.with_extension("");
    fs::write(&asm_file, assembly_code)?;

    // Invoke Assembler
    let assembler_status = Command::new("clang")
        .arg("-target")
        .arg("x86_64-apple-darwin")
        .arg(&asm_file)
        .arg("-o")
        .arg(&exec_file)
        .status()
        .expect("failed to run clang");
    if !assembler_status.success() {
        return Err("clang failed to assemble and link".into());
    }

    fs::remove_file(&asm_file)?;
    return Ok(());
}
