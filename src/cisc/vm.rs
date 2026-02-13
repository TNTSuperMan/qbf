use crate::{cisc::bytecode::Bytecode, memory::Memory, trace::OperationCountMap};

pub struct VM<'a> {
    pub insts: &'a [Bytecode],
    pub memory: Memory,
    pub ocm: OperationCountMap,
    pub pc: usize,
    pub pointer: usize,
    pub flush: bool,
}

impl<'a> VM<'a> {
    pub fn new(bytecodes: &'a [Bytecode], flush: bool) -> Result<VM<'a>, String> {
        let ocm = OperationCountMap::new(bytecodes.len());
        Ok(VM {
            insts: bytecodes,
            memory: Memory::new(),
            ocm,
            pc: 0,
            pointer: 0,
            flush,
        })
    }
    pub fn step_ptr(&mut self, delta: isize) {
        self.pointer = self.pointer.wrapping_add_signed(delta);
    }
}

pub struct UnsafeVM<'a, 'b> {
    pub inner: &'b mut VM<'a>,
    memory_at: *mut u8,
    pointer: *mut u8,
    internal_insts_at: *const Bytecode,
    internal_pc: *const Bytecode,
}
impl<'a, 'b> UnsafeVM<'a, 'b> {
    pub unsafe fn new(vm: &'b mut VM<'a>) -> UnsafeVM<'a, 'b> {
        let memory_at = vm.memory.0.as_mut_ptr();
        let pointer = memory_at.add(vm.pointer);
        let internal_insts_at = vm.insts.as_ptr();
        let internal_pc = internal_insts_at.add(vm.pc);
        UnsafeVM { inner: vm, memory_at, pointer, internal_insts_at, internal_pc }
    }

    pub fn get_pc(&self) -> usize {
        // SAFETY: 差分を求めるだけだから安全なはず
        unsafe { self.internal_pc.offset_from_unsigned(self.internal_insts_at) }
    }
    pub unsafe fn get_op(&mut self) -> &Bytecode {
        #[cfg(feature = "debug")] {
            if self.get_pc() >= self.inner.insts.len() {
                panic!("[UNSAFE] Runtime Error: Out of range insts");
            }
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
        #[cfg(feature = "debug")] { self.rangecheck(0); }
        *self.pointer
    }
    pub unsafe fn set(&mut self, value: u8) {
        #[cfg(feature = "debug")] { self.rangecheck(0); }
        *self.pointer = value;
    }
    pub unsafe fn add(&mut self, value: u8) {
        #[cfg(feature = "debug")] { self.rangecheck(0); }
        *self.pointer = (*self.pointer).wrapping_add(value);
    }
    pub unsafe fn sub(&mut self, value: u8) {
        #[cfg(feature = "debug")] { self.rangecheck(0); }
        *self.pointer = (*self.pointer).wrapping_sub(value);
    }

    pub unsafe fn get_with_offset(&self, offset: isize) -> u8 {
        #[cfg(feature = "debug")] { self.rangecheck(offset); }
        *(self.pointer.wrapping_add(offset as usize))
    }
    pub unsafe fn set_with_offset(&mut self, offset: isize, value: u8) {
        #[cfg(feature = "debug")] { self.rangecheck(offset); }
        *(self.pointer.wrapping_add(offset as usize)) = value;
    }
    pub unsafe fn add_with_offset(&mut self, offset: isize, value: u8) {
        #[cfg(feature = "debug")] { self.rangecheck(offset); }
        let p = self.pointer.wrapping_add(offset as usize);
        *p = (*p).wrapping_add(value);
    }
    pub unsafe fn sub_with_offset(&mut self, offset: isize, value: u8) {
        #[cfg(feature = "debug")] { self.rangecheck(offset); }
        let p = self.pointer.wrapping_add(offset as usize);
        *p = (*p).wrapping_sub(value);
    }
}
impl<'a, 'b> Drop for UnsafeVM<'a, 'b> {
    fn drop(&mut self) {
        self.inner.pointer = self.get_ptr();
        self.inner.pc = self.get_pc();
    }
}
