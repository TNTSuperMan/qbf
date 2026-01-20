const MEMORY_LENGTH: usize = 65536;

pub struct Memory(pub Box<[u8; MEMORY_LENGTH]>);

impl Memory {
    pub fn new() -> Memory {
        Memory(Box::new([0; MEMORY_LENGTH]))
    }
}

impl Memory {
    pub fn get(&self, index: isize) -> Result<u8, String> {
        match self.0.get(index as usize) {
            Some(val) => Ok(*val),
            None => Err(format!("Runtime Error: Out of range memory get, Address: {}", index)),
        }
    }
    pub fn set(&mut self, index: isize, value: u8) -> Result<(), String> {
        if self.0.len() <= index as usize {
            Err(format!("Runtime Error: Out of range memory set, Address: {}", index))
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                *self.0.as_mut_ptr().add(index as usize) = value;
            }
            Ok(())
        }
    }
    pub fn add(&mut self, index: isize, value: u8) -> Result<(), String> {
        if self.0.len() <= index as usize {
            Err(format!("Runtime Error: Out of range memory add, Address: {}", index))
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                let ptr = self.0.as_mut_ptr().add(index as usize);
                *ptr = (*ptr).wrapping_add(value);
            }
            Ok(())
        }
    }
    pub fn sub(&mut self, index: isize, value: u8) -> Result<(), String> {
        if self.0.len() <= index as usize {
            Err(format!("Runtime Error: Out of range memory sub, Address: {}", index))
        } else {
            unsafe { // SAFETY: 直前に範囲を確認済み
                let ptr = self.0.as_mut_ptr().add(index as usize);
                *ptr = (*ptr).wrapping_sub(value);
            }
            Ok(())
        }
    }
    #[inline(always)]
    pub unsafe fn get_unchecked(&self, index: usize) -> u8 {
        #[cfg(feature = "debug")] {
            if self.0.len() <= index as usize {
                panic!("[UNSAFE] Runtime Error: Out of range memory get, Address: {} ", index);
            }
        }
        *self.0.as_ptr().add(index)
    }
    #[inline(always)]
    pub unsafe fn set_unchecked(&mut self, index: usize, value: u8) {
        #[cfg(feature = "debug")] {
            if self.0.len() <= index as usize {
                panic!("[UNSAFE] Runtime Error: Out of range memory set, Address: {} ", index);
            }
        }
        *self.0.as_mut_ptr().add(index) = value;
    }
    #[inline(always)]
    pub unsafe fn add_unchecked(&mut self, index: usize, value: u8) {
        #[cfg(feature = "debug")] {
            if self.0.len() <= index as usize {
                panic!("[UNSAFE] Runtime Error: Out of range memory add, Address: {} ", index);
            }
        }
        let ptr = self.0.as_mut_ptr().add(index);
        *ptr = (*ptr).wrapping_add(value);
    }
    #[inline(always)]
    pub unsafe fn sub_unchecked(&mut self, index: usize, value: u8) {
        #[cfg(feature = "debug")] {
            if self.0.len() <= index as usize {
                panic!("[UNSAFE] Runtime Error: Out of range memory sub, Address: {} ", index);
            }
        }
        let ptr = self.0.as_mut_ptr().add(index);
        *ptr = (*ptr).wrapping_sub(value);
    }
}
