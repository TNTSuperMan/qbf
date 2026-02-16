use crate::cisc::{vm::Program, bytecode::Bytecode};

#[cfg(feature = "debug")]
use crate::trace::OperationCountMap;

#[cfg(feature = "debug")]
pub fn generate_bytecode_trace(bytecodes: &[Bytecode], ocm: &OperationCountMap) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in bytecodes.iter().enumerate() {
        match b {
            Bytecode::JmpIfNotZero { .. } => lv -= 1,
            Bytecode::PositiveRangeCheckJNZ { .. } => lv -= 1,
            Bytecode::NegativeRangeCheckJNZ { .. } => lv -= 1,
            Bytecode::BothRangeCheckJNZ { .. } => lv -= 1,
            _ => {}
        }
        str += &format!("{}\t{}\t{}{:?}\n", (ocm.deopt[i].wrapping_add(1) as f64).log2().floor(), (ocm.opt[i].wrapping_add(1) as f64).log2().floor(), "    ".repeat(lv), b);
        match b {
            Bytecode::JmpIfZero { .. } => lv += 1,
            _ => {}
        }
    }
    str += &format!("step count(deopt/opt): {}/{}", ocm.deopt.iter().fold(0, |acc, e| acc + e), ocm.opt.iter().fold(0, |acc, e| acc + e));

    str
}

#[cfg(not(feature = "debug"))]
pub fn write_trace(vm: &Program, insts: &[Bytecode]) {}

#[cfg(feature = "debug")]
pub fn write_trace(vm: &Program, insts: &[Bytecode]) {
    use std::fs;
    use crate::cisc::trace::generate_bytecode_trace;

    fs::write("./box/bytecodes", generate_bytecode_trace(&insts, &vm.ocm)).expect("failed to write");
    //fs::write("./box/memory", *vm.memory.0).expect("failed to write");
}
