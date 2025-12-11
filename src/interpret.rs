use std::{io::{Write, stdout}};

use crate::{bytecode::{Bytecode, OpCode}, memory::Memory, trace::OperationCountMap};

pub fn run(insts: Vec<Bytecode>, memory: &mut impl Memory, map: &mut OperationCountMap) -> Result<(), String> {
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
                let add_res = memory.get(ptr)?.wrapping_add(bytecode.val);
                memory.set(ptr, add_res)?;
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
                let mul_val = memory.get(ptr)?.wrapping_add(
                    mul_cache.wrapping_mul(bytecode.val)
                );
                memory.set(ptr, mul_val)?;
            }
            OpCode::AddFromMemory => {
                let add_val = memory.get(ptr)?.wrapping_add(mul_cache);
                memory.set(ptr, add_val)?;
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
