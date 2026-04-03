pub mod asm;
pub mod codegen;
pub mod emit;
pub mod fixup;

use std::str::FromStr;

use crate::frontend::Stage;
use crate::frontend::ir;

#[derive(Debug, Clone, Copy)]
pub enum Target {
    Intel,
    Arm,
    RiscV,
}

impl FromStr for Target {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "x86_64" | "x64" | "intel" | "amd64" => Ok(Target::Intel),
            "aarch64" | "arm64" | "arm" => Ok(Target::Arm),
            "riscv64" | "riscv" => Ok(Target::RiscV),
            _ => Err(format!("Unsupported target architecture: {}", s)),
        }
    }
}

pub fn compile(ir: ir::Program, target: Target, stage: Stage) -> Option<String> {
    match target {
        Target::Intel => {
            let ast = codegen::generate(ir);
            if stage == Stage::Codegen {
                dbg!(&ast);
                println!("Code Generation OK!");
                return None;
            }

            let assembly_code = emit::emit(ast);
            if stage == Stage::Emit {
                println!("{}", assembly_code);
                println!("Code Emission OK!");
                return None;
            }

            return Some(assembly_code);
        }
        Target::Arm => unimplemented!(),
        Target::RiscV => unimplemented!(),
    }
}
