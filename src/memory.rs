pub trait Memory {
    fn get(&self, index: isize) -> Result<u8, String>;
    fn set(&mut self, index: isize, value: u8) -> Result<(), String>;
    fn add(&mut self, index: isize, value: u8) -> Result<(), String>;
    unsafe fn get_unchecked(&self, index: usize) -> u8;
    unsafe fn set_unchecked(&mut self, index: usize, value: u8);
}

// TODO: DynamicMemoryを追加

pub struct StaticMemory(pub Box<[u8; 65536]>);

impl StaticMemory {
    pub fn new() -> StaticMemory {
        StaticMemory(Box::new([0; 65536]))
    }
}

impl Memory for StaticMemory {
    fn get(&self, index: isize) -> Result<u8, String> {
        match self.0.get(index as usize) {
            Some(val) => Ok(*val),
            None => Err(format!("Runtime Error: Out of range memory read, Address: {}", index)),
        }
    }
    fn set(&mut self, index: isize, value: u8) -> Result<(), String> {
        match self.0.get_mut(index as usize) {
            Some(val) => {
                *val = value;
                Ok(())
            },
            None => Err(format!("Runtime Error: Out of range memory write, Address: {}", index)),
        }
    }
    fn add(&mut self, index: isize, value: u8) -> Result<(), String> {
        if self.0.len() <= index as usize {
            Err(format!("Runtime Error: Out of range memory read, Address: {}", index))
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                let ptr = self.0.as_mut_ptr().add(index as usize);
                *ptr = (*ptr).wrapping_add(value);
            }
            Ok(())
        }
    }
    unsafe fn get_unchecked(&self, index: usize) -> u8 {
        *self.0.get_unchecked(index)
    }
    unsafe fn set_unchecked(&mut self, index: usize, value: u8) {
        *self.0.get_unchecked_mut(index) = value;
    }
}
