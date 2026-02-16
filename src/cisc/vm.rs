use crate::{cisc::bytecode::Bytecode, trace::OperationCountMap};

pub struct Program<'a> {
    pub ocm: OperationCountMap,
    insts: &'a [Bytecode],
    pc: usize,
    pub flush: bool,
}

impl<'a> Program<'a> {
    pub fn new(bytecodes: &[Bytecode], flush: bool) -> Program {
        let ocm = OperationCountMap::new(bytecodes.len());
        Program {
            ocm,
            insts: bytecodes,
            pc: 0,
            flush,
        }
    }
    pub fn pc(&self) -> usize {
        self.pc
    }
    pub fn inst(&self) -> &Bytecode {
        &self.insts[self.pc]
    }
    pub fn step(&mut self) {
        self.pc = self.pc.wrapping_add(1);
    }
    pub fn jump_abs(&mut self, addr: usize) {
        self.pc = addr as usize;
    }
    pub fn jump_back(&mut self, addr: usize) {
        self.pc = self.pc.wrapping_sub(addr);
    }
}


pub struct UnsafeProgram<'a, 'b> {
    pub inner: &'b mut Program<'a>,
    insts_len: usize,
    internal_insts_at: *const Bytecode,
    internal_pc: *const Bytecode,
}
impl<'a, 'b> UnsafeProgram<'a, 'b> {
    pub unsafe fn new(program: &'b mut Program<'a>) -> UnsafeProgram<'a, 'b> {
        let insts_len = program.insts.len();
        let internal_insts_at = program.insts.as_ptr();
        let pc = program.pc();
        UnsafeProgram {
            inner: program,
            insts_len,
            internal_insts_at,
            internal_pc: internal_insts_at.add(pc),
        }
    }

    pub fn pc(&self) -> usize {
        // SAFETY: 差分を求めるだけだから安全なはず
        unsafe { self.internal_pc.offset_from_unsigned(self.internal_insts_at) }
    }
    pub unsafe fn inst(&self) -> &Bytecode {
        if cfg!(feature = "debug") && self.pc() >= self.insts_len {
            panic!("[UNSAFE] Runtime Error: Out of range insts");
        }
        &*self.internal_pc
    }

    pub unsafe fn jump_abs(&mut self, to: u32) {
        self.internal_pc = self.internal_insts_at.add(to as usize);
    }
    pub unsafe fn jump_back(&mut self, to: u16) {
        self.internal_pc = self.internal_pc.sub(to as usize);
    }
    pub unsafe fn jump_one(&mut self) {
        self.internal_pc = self.internal_pc.add(1);
    }
}
impl<'a, 'b> Drop for UnsafeProgram<'a, 'b> {
    fn drop(&mut self) {
        self.inner.pc = self.pc();
    }
}
