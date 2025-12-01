use std::{io::{Write, stdout}};

use crate::{bytecode::{Bytecode, OpCode}, trace::OperationCountMap};

pub fn run(insts: Vec<Bytecode>, size: usize, map: &mut OperationCountMap) -> Result<Vec<u8>, String> {
    let mut stdout = stdout().lock();
    let mut pc: usize = 0;
    let mut offset: isize = 0;
    let mut memory: Vec<u8> = vec![0; size];
    let mut mul_cache: u8 = 0;

    macro_rules! get {
        ($index: expr) => {
            if let Some(value) = memory.get($index as usize) {
                Ok(*value)
            } else {
                Err(format!("Out of range memory read. Address: {}, PC: {}", $index, pc))
            }
        };
    }
    macro_rules! set {
        ($index: expr, $value: expr) => {
            if let Some(mem) = memory.get_mut($index as usize) {
                *mem = $value;
                Ok(())
            } else {
                Err(format!("Out of range memory write. Address: {}, PC: {}", $index, pc))
            }
        };
    }

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
                let add_res = get!(ptr)?.wrapping_add(bytecode.val);
                set!(ptr, add_res)?;
            }
            OpCode::Set => {
                set!(ptr, bytecode.val)?;
            }

            OpCode::Shift => {
                while get!(offset.wrapping_add(bytecode.ptr))? != 0 {
                    offset = offset.wrapping_add(bytecode.ptr2);
                }
            }
            OpCode::MulStart => {
                mul_cache = get!(ptr)?;
                if mul_cache == 0 {
                    pc = bytecode.addr;
                    continue;
                } else {
                    set!(offset.wrapping_add(bytecode.ptr2), 0)?;
                }
            }
            OpCode::Mul => {
                let mul_val = get!(ptr)?.wrapping_add(
                    mul_cache.wrapping_mul(bytecode.val)
                );
                set!(ptr, mul_val)?;
            }

            OpCode::In => {
                set!(ptr, 0)?; // TODO
            }
            OpCode::Out => {
                stdout.write(&[get!(ptr)?]).unwrap();
            }

            OpCode::LoopStart => {
                if get!(ptr)? == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::LoopEnd => {
                if get!(ptr)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::LoopEndWithOffset => {
                offset = offset.wrapping_add(bytecode.ptr2);
                if get!(ptr)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::End => {
                return Ok(memory);
            }
        }
        pc = pc.wrapping_add(1);
    }
}
