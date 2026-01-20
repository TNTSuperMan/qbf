use crate::{cisc::{interpret::run, vm::VM}, ir::IR, range::RangeInfo};

mod bytecode;
mod interpret;
mod trace;
mod vm;

pub fn run_cisc(ir_nodes: &[IR], range_info: &RangeInfo) -> Result<(), String> {
    let mut vm = VM::new(ir_nodes, range_info)?;
    let result = run(&mut vm);
    
    #[cfg(feature = "debug")] {
        use std::fs;
        use crate::cisc::trace::generate_bytecode_trace;

        fs::write("./box/bytecodes", generate_bytecode_trace(&vm.insts, &vm.ocm)).expect("failed to write");
        fs::write("./box/memory", *vm.memory.0).expect("failed to write");
    }

    result
}
