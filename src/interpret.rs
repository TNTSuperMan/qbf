use std::io::{Write, stdout};

use crate::{parser::{InstOp, Instruction}};

pub fn run(insts: Vec<Instruction>, size: usize) {
    let mut stdout = stdout().lock();
    let mut pc: usize = 0;
    let mut offset: isize = 0;
    let mut memory: Vec<u8> = vec![0; size];
    loop {
        let Instruction { opcode, pointer } = &insts[pc];
        let ptr = offset.wrapping_add(*pointer) as usize;
        match opcode {
            InstOp::Breakpoint => {
                // 標準出力と分けるだけ、エラーじゃない
                eprintln!("PC: {}, PTR: {}, ", pc, ptr);
            }

            InstOp::Add(val) => {
                memory[ptr] = memory[ptr].wrapping_add(*val);
            }
            InstOp::Set(val) => {
                memory[ptr] = *val;
            }

            InstOp::Shift(diff) => {
                while memory[offset.wrapping_add(*pointer) as usize] != 0 {
                    offset = offset.wrapping_add(*diff);
                }
            }
            InstOp::MulAndSetZero(dests) => {
                let source_val = memory[ptr];
                if source_val != 0 {
                    for (dest_p, m) in dests {
                        let dest_ptr = offset.wrapping_add(*dest_p) as usize;
                        memory[dest_ptr] = memory[dest_ptr].wrapping_add(source_val.wrapping_mul(*m));
                    }
                    memory[ptr] = 0;
                }
            }
            InstOp::MulAndSetZeroTo(source, dests) => {
                let source_val = memory[offset.wrapping_add(*source) as usize].wrapping_add(memory[ptr]);
                if source_val != 0 {
                    for (dest_p, m) in dests {
                        let dest_ptr = offset.wrapping_add(*dest_p) as usize;
                        memory[dest_ptr] = memory[dest_ptr].wrapping_add(source_val.wrapping_mul(*m));
                    }
                    memory[ptr] = 0;
                }
            }

            InstOp::In => {
                memory[ptr] = 0; // TODO
            }
            InstOp::Out => {
                stdout.write(&[memory[ptr]]).unwrap();
            }

            InstOp::LoopStart(end) => {
                if memory[ptr] == 0 {
                    pc = *end;
                }
            }
            InstOp::LoopEnd(start) => {
                if memory[ptr] != 0 {
                    pc = *start;
                }
            }
            InstOp::LoopEndWithOffset(start, off) => {
                if memory[ptr] != 0 {
                    pc = *start;
                }
                offset = offset.wrapping_add(*off);
            }
            InstOp::End => {
                break;
            }
        }
        pc = pc.wrapping_add(1);
    }
}
