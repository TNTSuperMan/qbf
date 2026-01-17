use std::io::{Read, Write, stdin, stdout};

use crate::{bytecode::{Bytecode, OpCode}, memory::Memory, trace::OperationCountMap};

#[inline(always)]
pub fn u32_to_delta_and_val(val: u32) -> (i16, u8) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u8,
    )
}

pub fn run(insts: &[Bytecode], memory: &mut Memory, ocm: &mut OperationCountMap) -> Result<(), String> {
    let mut stdout = stdout().lock();
    let mut stdin = stdin().lock();
    let mut stdin_buf: [u8; 1] = [0];
    let mut pc: usize = 0;
    let mut pointer: isize = 0;
    let mut mul_val: u8 = 0;
    
    loop {
        #[cfg(feature = "debug")] {
            ocm.0[pc] += 1;
        }

        let bytecode = &insts[pc];
        pointer += bytecode.delta as isize;
        
        match bytecode.opcode {
            OpCode::Breakpoint => {
                eprintln!("PC: {}, PTR: {}", pc, pointer);
            }

            OpCode::SingleAdd => {
                memory.add(pointer, bytecode.val)?;
            }
            OpCode::SingleSet => {
                memory.set(pointer, bytecode.val)?;
            }
            OpCode::AddAdd => {
                memory.add(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode::AddSet => {
                memory.add(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }
            OpCode::SetAdd => {
                memory.set(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode::SetSet => {
                memory.set(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }

            OpCode::Shift => {
                let step = bytecode.addr as i32 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
            }
            OpCode::ShiftAdd => {
                let step = bytecode.val as i8 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode::ShiftSet => {
                let step = bytecode.val as i8 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }

            OpCode::MulStart => {
                let val = memory.get(pointer)?;
                if val == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::Mul => {
                memory.add(pointer, mul_val.wrapping_mul(bytecode.val))?;
            }
            OpCode::MulLast => {
                memory.add(pointer, mul_val.wrapping_mul(bytecode.val))?;
                pointer += bytecode.addr as i32 as isize;
            }

            OpCode::In => {
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => memory.set(pointer, stdin_buf[0])?,
                    Err(_) => memory.set(pointer, 0)?,
                }
            }
            OpCode::Out => {
                stdout.write(&[memory.get(pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode::JmpIfZero => {
                if memory.get(pointer)? == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::JmpIfNotZero => {
                if memory.get(pointer)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }

            OpCode::End => {
                return Ok(());
            }
        }
        pc += 1;
    }
}
