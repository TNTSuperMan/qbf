#[cfg(feature = "debug")]
use crate::{risc::bytecode::{Bytecode, OpCode}, trace::OperationCountMap};

#[cfg(feature = "debug")]
pub fn generate_bytecode_trace(bytecodes: &[Bytecode], ocm: &OperationCountMap) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in bytecodes.iter().enumerate() {
        if b.opcode == OpCode::JmpIfNotZero {
            lv -= 1;
        }
        str += &format!("{}:\t{}{:?}\n", (ocm.0[i].wrapping_add(1) as f64).log2().floor(), "    ".repeat(lv), b);
        if b.opcode == OpCode::JmpIfZero {
            lv += 1;
        }
    }

    str
}
