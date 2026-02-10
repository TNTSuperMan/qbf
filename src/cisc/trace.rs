use crate::cisc::vm::VM;

#[cfg(feature = "debug")]
use crate::{cisc::bytecode::NewBytecode, trace::OperationCountMap};

#[cfg(feature = "debug")]
pub fn generate_bytecode_trace(bytecodes: &[NewBytecode], ocm: &OperationCountMap) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in bytecodes.iter().enumerate() {
        match b {
            NewBytecode::JmpIfNotZero { .. } => lv -= 1,
            NewBytecode::PositiveRangeCheckJNZ { .. } => lv -= 1,
            NewBytecode::NegativeRangeCheckJNZ { .. } => lv -= 1,
            NewBytecode::BothRangeCheckJNZ { .. } => lv -= 1,
            _ => {}
        }
        str += &format!("{}\t{}\t{}{:?}\n", (ocm.deopt[i].wrapping_add(1) as f64).log2().floor(), (ocm.opt[i].wrapping_add(1) as f64).log2().floor(), "    ".repeat(lv), b);
        match b {
            NewBytecode::JmpIfZero { .. } => lv += 1,
            _ => {}
        }
    }
    str += &format!("step count(deopt/opt): {}/{}", ocm.deopt.iter().fold(0, |acc, e| acc + e), ocm.opt.iter().fold(0, |acc, e| acc + e));

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
