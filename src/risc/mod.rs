use crate::{risc::{bytecode::ir_to_bytecodes, interpret::run}, ir::IR, memory::Memory, trace::OperationCountMap};

mod bytecode;
mod interpret;
mod trace;

pub fn run_risc(ir_nodes: &[IR]) -> Result<(), String> {
    let bytecodes = ir_to_bytecodes(ir_nodes)?;
    let mut memory = Memory::new();
    let mut ocm = OperationCountMap::new(bytecodes.len());
    run(&bytecodes, &mut memory, &mut ocm)?;
    
    #[cfg(feature = "debug")] {
        use std::fs;
        use crate::risc::trace::generate_bytecode_trace;

        fs::write("./box/bytecodes", generate_bytecode_trace(&bytecodes, &ocm)).expect("failed to write");
        fs::write("./box/memory", *memory.0).expect("failed to write");
    }

    Ok(())
}
