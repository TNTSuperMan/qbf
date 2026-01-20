use crate::{cisc::bytecode::{Bytecode, ir_to_bytecodes}, ir::IR, memory::Memory, range::RangeInfo, trace::OperationCountMap};

pub struct VM {
    pub insts: Box<[Bytecode]>,
    pub memory: Memory,
    pub ocm: OperationCountMap,
    pub pc: usize,
    pub pointer: usize,
}

impl VM {
    pub fn new(ir: &[IR], range_info: &RangeInfo) -> Result<VM, String> {
        let bytecodes = ir_to_bytecodes(ir, range_info)?;
        let ocm = OperationCountMap::new(bytecodes.len());
        Ok(VM {
            insts: bytecodes.into_boxed_slice(),
            memory: Memory::new(),
            ocm,
            pc: 0,
            pointer: 0,
        })
    }
}
