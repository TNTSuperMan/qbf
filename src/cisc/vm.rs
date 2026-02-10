use crate::{cisc::bytecode::{NewBytecode, ir_to_bytecodes}, ir::IR, memory::Memory, range::RangeInfo, trace::OperationCountMap};

pub struct VM {
    pub insts: Box<[NewBytecode]>,
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
    pub fn step_ptr(&mut self, delta: isize) {
        self.pointer = self.pointer.wrapping_add_signed(delta);
    }
}

pub struct UnsafeVM<'a> {
    pub vm: &'a mut VM,
    pub pc: usize,
    pointer: *mut u8,
}
impl<'a> UnsafeVM<'a> {
    pub unsafe fn new(vm: &'a mut VM) -> UnsafeVM<'a> {
        let pointer = vm.memory.0.as_mut_ptr().add(vm.pointer);
        let pc = vm.pc;
        UnsafeVM { vm, pointer, pc }
    }

    pub unsafe fn get_op(&mut self) -> &NewBytecode {
        #[cfg(feature = "debug")] {
            if self.pc >= self.vm.insts.len() {
                panic!("[UNSAFE] Runtime Error: Out of range insts");
            }
        }
        self.vm.insts.get_unchecked(self.pc)
    }

    pub fn get_ptr(&self) -> usize {
        // SAFETY: 差分を求めるだけだから安全なはず
        unsafe { self.pointer.offset_from_unsigned(self.vm.memory.0.as_ptr()) }
    }

    pub fn rangecheck(&self, offset: isize) {
        if self.vm.memory.0.len() <= (self.get_ptr().wrapping_add_signed(offset)) {
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
impl<'a> Drop for UnsafeVM<'a> {
    fn drop(&mut self) {
        self.vm.pointer = self.get_ptr();
        self.vm.pc = self.pc;
    }
}
