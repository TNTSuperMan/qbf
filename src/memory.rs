use anyhow::{Context, Result};

const MEMORY_LENGTH: usize = 65536;

pub struct Memory(pub Box<[u8; MEMORY_LENGTH]>);

impl Memory {
    pub fn new() -> Memory {
        Memory(Box::new([0; MEMORY_LENGTH]))
    }
}

impl Memory {
    pub fn get(&self, index: usize) -> Result<u8> {
        Ok(*self.0.get(index).with_context(|| format!("Runtime Error: Out of range memory read, Address: {index}"))?)
    }
    pub fn set(&mut self, index: usize, value: u8) -> Result<()> {
        let mem = self.0.get_mut(index).with_context(|| format!("Runtime Error: Out of range memory write, Address: {index}"))?;
        *mem = value;
        Ok(())
    }
    pub fn add(&mut self, index: usize, value: u8) -> Result<()> {
        let mem = self.0.get_mut(index).with_context(|| format!("Runtime Error: Out of range memory add, Address: {index}"))?;
        *mem = mem.wrapping_add(value);
        Ok(())
    }
    pub fn sub(&mut self, index: usize, value: u8) -> Result<()> {
        let mem = self.0.get_mut(index).with_context(|| format!("Runtime Error: Out of range memory sub, Address: {index}"))?;
        *mem = mem.wrapping_sub(value);
        Ok(())
    }
}
