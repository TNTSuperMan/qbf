use anyhow::{Result, bail};

const MEMORY_LENGTH: usize = 65536;

pub struct Memory(pub Box<[u8; MEMORY_LENGTH]>);

impl Memory {
    pub fn new() -> Memory {
        Memory(Box::new([0; MEMORY_LENGTH]))
    }
}

impl Memory {
    pub fn get(&self, index: usize) -> Result<u8> {
        match self.0.get(index) {
            Some(val) => Ok(*val),
            None => bail!("Runtime Error: Out of range memory get, Address: {}", index),
        }
    }
    pub fn set(&mut self, index: usize, value: u8) -> Result<()> {
        if self.0.len() <= index {
            bail!("Runtime Error: Out of range memory set, Address: {}", index);
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                *self.0.as_mut_ptr().add(index) = value;
            }
            Ok(())
        }
    }
    pub fn add(&mut self, index: usize, value: u8) -> Result<()> {
        if self.0.len() <= index {
            bail!("Runtime Error: Out of range memory add, Address: {}", index);
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                let ptr = self.0.as_mut_ptr().add(index);
                *ptr = (*ptr).wrapping_add(value);
            }
            Ok(())
        }
    }
    pub fn sub(&mut self, index: usize, value: u8) -> Result<()> {
        if self.0.len() <= index {
            bail!("Runtime Error: Out of range memory sub, Address: {}", index);
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                let ptr = self.0.as_mut_ptr().add(index);
                *ptr = (*ptr).wrapping_sub(value);
            }
            Ok(())
        }
    }
}
