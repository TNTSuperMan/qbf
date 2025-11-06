use crate::{interpret::step, parser::{Instruction, parse}};

pub struct BFVM {
    pub insts: Vec<Instruction>,
    pub pc: usize,
    pub offset: isize,
    pub memory: Vec<u8>,
    pub output: Box<dyn Fn(u8)>,
    pub input: Box<dyn Fn() -> u8>,
}

impl BFVM {
    pub fn new(code: &str, mem_size: usize, output: Box<dyn Fn(u8)>, input: Box<dyn Fn() -> u8>) -> BFVM {
        BFVM {
            insts: parse(code),
            pc: 0,
            offset: 0,
            memory: vec![0; mem_size],
            output,
            input
        }
    }
    pub fn run(&mut self) {
        let len = self.insts.len();
        while self.pc < len {
            step(self);
        }
    }
}
