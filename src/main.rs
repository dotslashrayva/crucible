use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod backend;
mod frontend;

use backend::codegen::generate;
use backend::emit::emit;

use frontend::irgen::flatten;
use frontend::lexer::lex;
use frontend::parser::parse;
use frontend::resolve::resolve;

#[derive(Debug, PartialEq)]
enum Stage {
    Lex,
    Parse,
    Validate,
    Ir,
    Codegen,
    Emit,
    Full,
}

struct Config {
    target_stage: Stage,
    input_path: PathBuf,
}

impl Config {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut target_stage = Stage::Full;
        let mut input_path = None;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--lex" => target_stage = Stage::Lex,
                "--parse" => target_stage = Stage::Parse,
                "--validate" => target_stage = Stage::Validate,
                "--tacky" | "--ir" => target_stage = Stage::Ir,
                "--codegen" => target_stage = Stage::Codegen,
                "-S" | "--emit" => target_stage = Stage::Emit,

                // Catch unknown flags to prevent silent typos
                flag if flag.starts_with('-') => return Err(format!("Unknown flag: {}", flag)),

                // Anything else is treated as our input file
                file => {
                    if input_path.is_some() {
                        return Err("Multiple input files are not yet supported.".to_string());
                    }
                    input_path = Some(PathBuf::from(file));
                }
            }
        }

        let input_path = input_path.ok_or_else(|| {
            "Usage: crucible <flag> <source.c>\nFlags: [--lex | --parse | --validate | --ir | --codegen | --emit]".to_string()
        })?;

        Ok(Config {
            target_stage,
            input_path,
        })
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse(env::args().skip(1))?;
    let input = config.input_path;
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

    let tokens = lex(&source).map_err(|e| format!("Lexical error: {}", e))?;
    if config.target_stage == Stage::Lex {
        dbg!(tokens);
        println!("Lexer OK!");
        return Ok(());
    }

    let mut ast = parse(tokens).map_err(|e| format!("Syntax error: {}", e))?;
    if config.target_stage == Stage::Parse {
        dbg!(ast);
        println!("Parser OK!");
        return Ok(());
    }

    resolve(&mut ast).map_err(|e| format!("Semantic error: {}", e))?;
    if config.target_stage == Stage::Validate {
        dbg!(ast);
        println!("Validation OK!");
        return Ok(());
    }

    let ir = flatten(ast);
    if config.target_stage == Stage::Ir {
        dbg!(ir);
        println!("IR OK!");
        return Ok(());
    }

    let assembly = generate(ir);
    if config.target_stage == Stage::Codegen {
        dbg!(assembly);
        println!("Code Generation OK!");
        return Ok(());
    }

    let assembly_code = emit(assembly);
    if config.target_stage == Stage::Emit {
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
