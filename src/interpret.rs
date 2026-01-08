use std::{io::{Write, stdout}};

use crate::{bytecode::{Bytecode, OpCode}, memory::Memory, trace::OperationCountMap};

pub fn run(insts: &[Bytecode], memory: &mut Memory, map: &mut OperationCountMap) -> Result<(), String> {
    let mut stdout = stdout().lock();
    let mut pc: usize = 0;
    let mut offset: isize = 0;
    let mut mul_cache: u8 = 0;

    loop {
        #[cfg(feature = "debug")] {
            map.0[pc] += 1;
        }
        let bytecode = &insts[pc];
        let ptr = offset.wrapping_add(bytecode.ptr) as isize;
        match bytecode.opcode {
            OpCode::Breakpoint => {
                // 標準出力と分けるだけ、エラーじゃない
                eprintln!("PC: {}, PTR: {}, ", pc, ptr);
            }

            OpCode::Add => {
                memory.add(ptr, bytecode.val)?;
            }
            OpCode::Set => {
                memory.set(ptr, bytecode.val)?;
            }

            OpCode::Shift => {
                while memory.get(offset.wrapping_add(bytecode.ptr))? != 0 {
                    offset = offset.wrapping_add(bytecode.ptr2);
                }
            }
            OpCode::MulStart => {
                mul_cache = memory.get(ptr)?;
                if mul_cache == 0 {
                    pc = bytecode.addr;
                    continue;
                } else {
                    memory.set(ptr, 0)?;
                }
            }
            OpCode::Mul => {
                memory.add(ptr, mul_cache.wrapping_mul(bytecode.val))?;
            }
            OpCode::AddFromMemory => {
                memory.add(ptr, mul_cache)?;
            }
            OpCode::SingleMul => {
                let ptr2 = offset.wrapping_add(bytecode.ptr2);
                let ptr2_val = memory.get(ptr2)?;
                if ptr2_val != 0 {
                    memory.add(ptr, ptr2_val.wrapping_mul(bytecode.val))?;
                    memory.set(ptr2, 0)?;
                }
            }
            OpCode::SingleAddFM => {
                let ptr2 = offset.wrapping_add(bytecode.ptr2);
                let ptr2_val = memory.get(ptr2)?;
                if ptr2_val != 0 {
                    memory.add(ptr, ptr2_val)?;
                    memory.set(ptr2, 0)?;
                }
            }

            OpCode::In => {
                memory.set(ptr, 0)?; // TODO
            }
            OpCode::Out => {
                stdout.write(&[memory.get(ptr)?]).unwrap();
            }

            OpCode::LoopStart => {
                if memory.get(ptr)? == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::LoopEnd => {
                if memory.get(ptr)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::LoopEndWithOffset => {
                offset = offset.wrapping_add(bytecode.ptr2);
                if memory.get(ptr)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::End => {
                return Ok(());
            }
        }
        pc = pc.wrapping_add(1);
    }
}
