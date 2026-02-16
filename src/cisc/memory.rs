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
}

impl Tape {
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
    pub fn sub(&mut self, value: u8) -> Result<(), RuntimeError> {
        let cell = self.buffer.get_mut(self.data_pointer).ok_or_else(|| RuntimeError::OOBSub(self.data_pointer, value))?;
        Ok(*cell = cell.wrapping_sub(value))
    }

    pub fn get_with_offset(&self, delta: isize) -> Result<u8, RuntimeError> {
        self.buffer.get(self.data_pointer.wrapping_add_signed(delta)).ok_or_else(|| RuntimeError::OOBGet(self.data_pointer)).copied()
    }
    pub fn set_with_offset(&mut self, delta: isize, value: u8) -> Result<(), RuntimeError> {
        let ptr = self.data_pointer.wrapping_add_signed(delta);
        let cell = self.buffer.get_mut(ptr).ok_or_else(|| RuntimeError::OOBSet(ptr, value))?;
        Ok(*cell = value)
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
