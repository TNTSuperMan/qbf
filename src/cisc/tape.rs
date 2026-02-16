use crate::cisc::error::RuntimeError;

const MEMORY_LENGTH: usize = 65536;

pub struct Tape {
    pub buffer: Box<[u8; MEMORY_LENGTH]>,
    pub data_pointer: usize,
}
impl Tape {
    pub fn new() -> Tape {
        Tape {
            buffer: Box::new([0; MEMORY_LENGTH]),
            data_pointer: 0,
        }
    }
    pub fn get(&self) -> Result<u8, RuntimeError> {
        self.buffer.get(self.data_pointer).ok_or_else(|| RuntimeError::OOBGet(self.data_pointer)).copied()
    }
    pub fn set(&mut self, value: u8) -> Result<(), RuntimeError> {
        let cell = self.buffer.get_mut(self.data_pointer).ok_or_else(|| RuntimeError::OOBSet(self.data_pointer, value))?;
        Ok(*cell = value)
    }
    pub fn add(&mut self, value: u8) -> Result<(), RuntimeError> {
        let cell = self.buffer.get_mut(self.data_pointer).ok_or_else(|| RuntimeError::OOBAdd(self.data_pointer, value))?;
        Ok(*cell = cell.wrapping_add(value))
    }
    
    pub fn add_with_offset(&mut self, delta: isize, value: u8) -> Result<(), RuntimeError> {
        let ptr = self.data_pointer.wrapping_add_signed(delta);
        let cell = self.buffer.get_mut(ptr).ok_or_else(|| RuntimeError::OOBAdd(ptr, value))?;
        Ok(*cell = cell.wrapping_add(value))
    }
    pub fn sub_with_offset(&mut self, delta: isize, value: u8) -> Result<(), RuntimeError> {
        let ptr = self.data_pointer.wrapping_add_signed(delta);
        let cell = self.buffer.get_mut(ptr).ok_or_else(|| RuntimeError::OOBSub(ptr, value))?;
        Ok(*cell = cell.wrapping_sub(value))
    }

    pub fn step(&mut self, delta: isize) {
        self.data_pointer = self.data_pointer.wrapping_add_signed(delta);
    }
}

pub struct UnsafeTape<'a> {
    pub inner: &'a mut Tape,
    buffer_at: *mut u8,
    data_pointer: *mut u8,
}
impl<'a> UnsafeTape<'a> {
    pub unsafe fn new(tape: &'a mut Tape) -> UnsafeTape<'a> {
        let buffer_at = tape.buffer.as_mut_ptr();
        let data_pointer = buffer_at.add(tape.data_pointer);
        UnsafeTape { inner: tape, buffer_at, data_pointer }
    }

    pub fn get_ptr(&self) -> usize {
        self.data_pointer.addr().wrapping_sub(self.buffer_at.addr())
    }

    pub fn rangecheck(&self, offset: isize) {
        if MEMORY_LENGTH <= (self.get_ptr().wrapping_add_signed(offset)) {
            panic!("[UNSAFE] Runtime Error: Out of range memory operation. Address: {} ", self.get_ptr());
        }
    }

    pub unsafe fn step_ptr(&mut self, delta: isize) {
        self.data_pointer = self.data_pointer.wrapping_add(delta as usize);
    }

    pub fn get_safe(&self, abs_ptr: usize) -> Result<u8, RuntimeError> {
        self.inner.buffer.get(abs_ptr).ok_or_else(|| RuntimeError::OOBGet(abs_ptr)).copied()
    }
    pub unsafe fn get(&self) -> u8 {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.data_pointer
    }
    pub fn set_safe(&mut self, abs_ptr: usize, value: u8) -> Result<(), RuntimeError> {
        let cell = self.inner.buffer.get_mut(abs_ptr).ok_or_else(|| RuntimeError::OOBSet(abs_ptr, value))?;
        Ok(*cell = value)
    }
    pub unsafe fn set(&mut self, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.data_pointer = value;
    }
    pub fn add_safe(&mut self, abs_ptr: usize, value: u8) -> Result<(), RuntimeError> {
        let cell = self.inner.buffer.get_mut(abs_ptr).ok_or_else(|| RuntimeError::OOBSet(abs_ptr, value))?;
        Ok(*cell = cell.wrapping_add(value))
    }
    pub unsafe fn add(&mut self, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(0); }
        *self.data_pointer = (*self.data_pointer).wrapping_add(value);
    }
    pub unsafe fn add_with_offset(&mut self, offset: isize, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(offset); }
        let p = self.data_pointer.wrapping_add(offset as usize);
        *p = (*p).wrapping_add(value);
    }
    pub unsafe fn sub_with_offset(&mut self, offset: isize, value: u8) {
        if cfg!(feature = "debug") { self.rangecheck(offset); }
        let p = self.data_pointer.wrapping_add(offset as usize);
        *p = (*p).wrapping_sub(value);
    }
}
impl<'a> Drop for UnsafeTape<'a> {
    fn drop(&mut self) {
        self.inner.data_pointer = self.get_ptr();
    }
}
