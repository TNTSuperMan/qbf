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

pub struct UnsafeVM<'a> {
    pub inner: &'a mut Program,
    memory_at: *mut u8,
    pointer: *mut u8,
}
#[allow(unused)]
impl<'a> UnsafeVM<'a> {
    pub unsafe fn new(vm: &'a mut Program) -> UnsafeVM<'a> {
        let memory_at = vm.memory.0.as_mut_ptr();
        let pointer = memory_at.add(vm.pointer);
        UnsafeVM { inner: vm, memory_at, pointer }
    }

    pub fn get_ptr(&self) -> usize {
        self.pointer.addr().wrapping_sub(self.memory_at.addr())
    }

    pub fn rangecheck(&self, offset: isize) {
        if self.inner.memory.0.len() <= (self.get_ptr().wrapping_add_signed(offset)) {
            panic!("[UNSAFE] Runtime Error: Out of range memory operation. Address: {} ", self.get_ptr());
        }
    }

    pub unsafe fn step_ptr(&mut self, delta: isize) {
        self.pointer = self.pointer.wrapping_add(delta as usize);
    }
    pub unsafe fn get(&self) -> u8 {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.pointer
    }
    pub unsafe fn set(&mut self, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.pointer = value;
    }
    pub unsafe fn add(&mut self, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.pointer = (*self.pointer).wrapping_add(value);
    }
    pub unsafe fn sub(&mut self, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.pointer = (*self.pointer).wrapping_sub(value);
    }

    pub unsafe fn get_with_offset(&self, offset: isize) -> u8 {
        if cfg!(feature = "debug") { self.rangecheck(offset); }
        *(self.pointer.wrapping_add(offset as usize))
    }
    pub unsafe fn set_with_offset(&mut self, offset: isize, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(offset); }
        *(self.pointer.wrapping_add(offset as usize)) = value;
    }
    pub unsafe fn add_with_offset(&mut self, offset: isize, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(offset); }
        let p = self.pointer.wrapping_add(offset as usize);
        *p = (*p).wrapping_add(value);
    }
    pub unsafe fn sub_with_offset(&mut self, offset: isize, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(offset); }
        let p = self.pointer.wrapping_add(offset as usize);
        *p = (*p).wrapping_sub(value);
    }
}
impl<'a> Drop for UnsafeVM<'a> {
    fn drop(&mut self) {
        self.inner.pointer = self.get_ptr();
    }
}

pub struct UnsafeInsts {
    insts_len: usize,
    internal_insts_at: *const Bytecode,
    internal_pc: *const Bytecode,
}
impl UnsafeInsts {
    pub unsafe fn new(insts: &[Bytecode], pc: usize) -> UnsafeInsts {
        let insts_len = insts.len();
        let internal_insts_at = insts.as_ptr();
        UnsafeInsts {
            insts_len,
            internal_insts_at,
            internal_pc: internal_insts_at.add(pc),
        }
    }

    pub fn get_pc(&self) -> usize {
        // SAFETY: 差分を求めるだけだから安全なはず
        unsafe { self.internal_pc.offset_from_unsigned(self.internal_insts_at) }
    }
    pub unsafe fn get_op(&self) -> &Bytecode {
        if cfg!(feature = "debug") && self.get_pc() >= self.insts_len {
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
