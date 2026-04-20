mod asm;
mod codegen;
mod emit;
mod fixup;

use crate::Stage;
use crate::frontend::ir;

use codegen::generate;
use emit::emit;

pub fn compile(ir_program: ir::Program, stage: Stage) -> Option<String> {
    // Code Generation
    let assembly = generate(ir_program);
    if stage == Stage::Codegen {
        dbg!(assembly);
        println!("Code Generation OK!");
        return None;
    }

    // Code Emission
    let assembly_code = emit(assembly);
    if stage == Stage::Emit {
        println!("{}", assembly_code);
        println!("Code Emission OK!");
        return None;
    }

    return Some(assembly_code);
}
