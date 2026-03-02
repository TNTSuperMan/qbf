use crate::{bytecode::bytecode::Bytecode, error::RuntimeError, trace::OperationCountMap};

pub struct Program<I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
{
    pub ocm: OperationCountMap,
    insts: Box<[Bytecode]>,
    pc: usize,
    pub step_remains: Option<usize>,
    input_fn: I,
    output_fn: O,
    io_break: bool,
}
impl<I, O> Program<I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
{
    pub fn new(bytecodes: Box<[Bytecode]>, timeout: Option<usize>, input_fn: I, output_fn: O, io_break: bool) -> Program<I, O> {
        let ocm = OperationCountMap::new(bytecodes.len());
        Program {
            ocm,
            insts: bytecodes,
            pc: 0,
            step_remains: timeout,
            input_fn, output_fn, io_break,
        }
    }
    pub fn check_timeout(&mut self) -> Result<(), RuntimeError> {
        if let Some(rem) = self.step_remains.as_mut() {
            *rem = rem.checked_sub(1).ok_or_else(|| RuntimeError::TimeoutError)?;
        }
        Ok(())
    }
    pub fn pc(&self) -> usize {
        self.pc
    }
    pub fn insts(&self) -> &[Bytecode] {
        &self.insts
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
    pub fn input(&mut self) -> u8 {
        (self.input_fn)()
    }
    pub fn output(&mut self, value: u8) {
        (self.output_fn)(value)
    }
    pub fn io_break(&self) -> bool {
        self.io_break
    }
}

pub struct UnsafeProgram<'a, I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
 {
    pub inner: &'a mut Program<I, O>,
    insts_len: usize,
    internal_insts_at: *const Bytecode,
    internal_pc: *const Bytecode,
}
#[allow(unsafe_op_in_unsafe_fn)]
impl<'a, I, O> UnsafeProgram<'a, I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
 {
    pub unsafe fn new(program: &'a mut Program<I, O>) -> UnsafeProgram<'a, I, O> {
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

    pub fn check_timeout(&mut self) -> Result<(), RuntimeError> {
        self.inner.check_timeout()
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
impl<'a, I, O> Drop for UnsafeProgram<'a, I, O>
where I: FnMut() -> u8,
      O: FnMut(u8) -> (),
 {
    fn drop(&mut self) {
        self.inner.pc = self.pc();
    }
}
