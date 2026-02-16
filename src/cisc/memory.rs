use crate::cisc::error::RuntimeError;

const MEMORY_LENGTH: usize = 65536;

pub struct Memory(pub Box<[u8; MEMORY_LENGTH]>);

impl Memory {
    pub fn new() -> Memory {
        Memory(Box::new([0; MEMORY_LENGTH]))
    }
}

impl Memory {
    pub fn get(&self, index: usize) -> Result<u8, RuntimeError> {
        self.0.get(index).ok_or_else(|| RuntimeError::OOBGet(index)).copied()
    }
    pub fn set(&mut self, index: usize, value: u8) -> Result<(), RuntimeError> {
        let cell = self.0.get_mut(index).ok_or_else(|| RuntimeError::OOBSet(index, value))?;
        Ok(*cell = value)
    }
    pub fn add(&mut self, index: usize, value: u8) -> Result<(), RuntimeError> {
        let cell = self.0.get_mut(index).ok_or_else(|| RuntimeError::OOBAdd(index, value))?;
        Ok(*cell = cell.wrapping_add(value))
    }
    pub fn sub(&mut self, index: usize, value: u8) -> Result<(), RuntimeError> {
        let cell = self.0.get_mut(index).ok_or_else(|| RuntimeError::OOBSub(index, value))?;
        Ok(*cell = cell.wrapping_sub(value))
    }
}
