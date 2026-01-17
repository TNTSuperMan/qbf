use crate::bytecode::Bytecode;
#[cfg(feature = "debug")]
use crate::ir::IR;

#[cfg(feature = "debug")]
pub struct OperationCountMap (pub Vec<usize>);
#[cfg(feature = "debug")]
impl OperationCountMap {
    pub fn new(len: usize) -> OperationCountMap {
        OperationCountMap(vec![0usize; len])
    }
}

#[cfg(not(feature = "debug"))]
pub struct OperationCountMap;
#[cfg(not(feature = "debug"))]
impl OperationCountMap {
    pub fn new(_len: usize) -> OperationCountMap {
        OperationCountMap
    }
}

#[cfg(feature = "debug")]
pub fn generate_bytecode_trace(bytecodes: &[Bytecode], ocm: &OperationCountMap) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for (i, b) in bytecodes.iter().enumerate() {
        use crate::bytecode::OpCode;

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

#[cfg(feature = "debug")]
pub fn generate_ir_trace(ir_nodes: &[IR]) -> String {
    let mut str = String::new();
    let mut lv: usize = 0;

    for ir in ir_nodes {
        use crate::ir::IROp;

        if let IROp::LoopEnd(_) = ir.opcode {
            lv -= 1;
        }
        if let IROp::LoopEndWithOffset(_, _) = ir.opcode {
            lv -= 1;
        }
        str += &format!("{}{} {:?}\n", "    ".repeat(lv), ir.pointer, ir.opcode);
        if let IROp::LoopStart(_) = ir.opcode {
            lv += 1;
        }
    }

    str
}
