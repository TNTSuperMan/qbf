#[cfg(feature = "debug")]
use std::io;

use crate::cisc::{program::Program, bytecode::Bytecode};

#[cfg(feature = "debug")]
use crate::cisc::tape::Tape;

#[cfg(feature = "debug")]
pub fn generate_bytecode_trace(program: &Program) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in program.insts().iter().enumerate() {
        match b {
            Bytecode::JmpIfNotZero { .. } => lv -= 1,
            Bytecode::PositiveRangeCheckJNZ { .. } => lv -= 1,
            Bytecode::NegativeRangeCheckJNZ { .. } => lv -= 1,
            Bytecode::BothRangeCheckJNZ { .. } => lv -= 1,
            _ => {}
        }
        str += &format!("{}\t{}\t{}{:?}\n", (program.ocm.deopt[i].wrapping_add(1) as f64).log2().floor(), (program.ocm.opt[i].wrapping_add(1) as f64).log2().floor(), "    ".repeat(lv), b);
        match b {
            Bytecode::JmpIfZero { .. } => lv += 1,
            _ => {}
        }
    }
    str += &format!("step count(deopt/opt): {}/{}", program.ocm.deopt.iter().fold(0, |acc, e| acc + e), program.ocm.opt.iter().fold(0, |acc, e| acc + e));

    str
}

#[cfg(not(feature = "debug"))]
pub fn write_trace(tape: &Tape, program: &Program) {}

#[cfg(feature = "debug")]
pub fn write_trace(tape: &Tape, program: &Program) -> Result<(), io::Error> {
    use std::fs;
    use crate::cisc::trace::generate_bytecode_trace;

    fs::write("./box/memory", *tape.buffer)?;
    fs::write("./box/bytecodes", generate_bytecode_trace(&program))?;

    Ok(())
}
