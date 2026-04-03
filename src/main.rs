use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

mod backend;
mod frontend;

use backend::Target;
use frontend::Stage;

struct Config {
    target_stage: Stage,
    target_arch: Target,
    input_path: PathBuf,
}

impl Config {
    fn parse(mut args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut target_stage = Stage::Full;
        let mut input_path = None;
        let mut target_arch = Target::Intel;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--lex" => target_stage = Stage::Lex,
                "--parse" => target_stage = Stage::Parse,
                "--validate" => target_stage = Stage::Validate,
                "--tacky" | "--ir" => target_stage = Stage::Ir,
                "--codegen" => target_stage = Stage::Codegen,
                "-S" | "--emit" => target_stage = Stage::Emit,

                "--target" => {
                    let arch_str = args
                        .next()
                        .ok_or("Expected architecture name after --target")?;
                    target_arch = arch_str.parse::<Target>()?;
                }

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
            target_arch,
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

    let ir = match frontend::run_pipeline(&source, config.target_stage)? {
        Some(ir) => ir,
        None => return Ok(()), // Frontend stopped early (e.g., --lex or --parse)
    };

    let assembly_code = match backend::compile(ir, config.target_arch, config.target_stage) {
        Some(code) => code,
        None => return Ok(()),
    };

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
