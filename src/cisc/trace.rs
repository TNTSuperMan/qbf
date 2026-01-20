#[cfg(feature = "debug")]
use crate::{cisc::{bytecode::{Bytecode, OpCode}, vm::VM}, trace::OperationCountMap};

#[cfg(feature = "debug")]
pub fn generate_bytecode_trace(bytecodes: &[Bytecode], ocm: &OperationCountMap) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in bytecodes.iter().enumerate() {
        if b.opcode == OpCode::JmpIfNotZero {
            lv -= 1;
        }
        if b.opcode == OpCode::PositiveRangeCheckJNZ {
            lv -= 1;
        }
        if b.opcode == OpCode::NegativeRangeCheckJNZ {
            lv -= 1;
        }
        str += &format!("{}:\t{}{:?}\n", (ocm.0[i].wrapping_add(1) as f64).log2().floor(), "    ".repeat(lv), b);
        if b.opcode == OpCode::JmpIfZero {
            lv += 1;
        }
    }

    str
}

#[cfg(not(feature = "debug"))]
pub fn write_trace(vm: &VM) {}

#[cfg(feature = "debug")]
pub fn write_trace(vm: &VM) {
    use std::fs;
    use crate::cisc::trace::generate_bytecode_trace;

    fs::write("./box/bytecodes", generate_bytecode_trace(&vm.insts, &vm.ocm)).expect("failed to write");
    fs::write("./box/memory", *vm.memory.0).expect("failed to write");
}
