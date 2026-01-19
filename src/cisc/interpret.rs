use std::io::{Read, Write, stdin, stdout};

use crate::{cisc::bytecode::{Bytecode, OpCode}, memory::Memory, trace::OperationCountMap};

#[inline(always)]
pub fn u32_to_delta_and_val(val: u32) -> (i16, u8) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u8,
    )
}

#[inline(always)]
pub fn u32_to_two_delta(val: u32) -> (i16, i16) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u16 as i16,
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
        
        match bytecode.opcode {
            OpCode::Breakpoint => {
                pointer += bytecode.delta as isize;
                eprintln!("PC: {}, PTR: {}", pc, pointer);
            }

            OpCode::SingleAdd => {
                pointer += bytecode.delta as isize;
                memory.add(pointer, bytecode.val)?;
            }
            OpCode::SingleSet => {
                pointer += bytecode.delta as isize;
                memory.set(pointer, bytecode.val)?;
            }
            OpCode::AddAdd => {
                pointer += bytecode.delta as isize;
                memory.add(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode::AddSet => {
                pointer += bytecode.delta as isize;
                memory.add(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }
            OpCode::SetAdd => {
                pointer += bytecode.delta as isize;
                memory.set(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode::SetSet => {
                pointer += bytecode.delta as isize;
                memory.set(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }

            OpCode::Shift => {
                pointer += bytecode.delta as isize;
                let step = bytecode.addr as i32 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
            }
            OpCode::ShiftAdd => {
                pointer += bytecode.delta as isize;
                let step = bytecode.val as i8 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode::ShiftSet => {
                pointer += bytecode.delta as isize;
                let step = bytecode.val as i8 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }

            OpCode::MulStart => {
                pointer += bytecode.delta as isize;
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
                memory.add(pointer + bytecode.delta as isize, mul_val.wrapping_mul(bytecode.val))?;
            }

            OpCode::SingleMoveAdd => {
                pointer += bytecode.delta as isize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    memory.set(pointer, 0)?;
                    memory.add(pointer + (bytecode.addr as i32 as isize), v)?;
                }
            }
            OpCode::SingleMoveSub => {
                pointer += bytecode.delta as isize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    memory.set(pointer, 0)?;
                    memory.sub(pointer + (bytecode.addr as i32 as isize), v)?;
                }
            }

            OpCode::DoubleMoveAddAdd => {
                pointer += bytecode.delta as isize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    memory.add(pointer + d1 as isize, v)?;
                    memory.add(pointer + d2 as isize, v)?;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::DoubleMoveAddSub => {
                pointer += bytecode.delta as isize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    memory.add(pointer + d1 as isize, v)?;
                    memory.sub(pointer + d2 as isize, v)?;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::DoubleMoveSubAdd => {
                pointer += bytecode.delta as isize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    memory.sub(pointer + d1 as isize, v)?;
                    memory.add(pointer + d2 as isize, v)?;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::DoubleMoveSubSub => {
                pointer += bytecode.delta as isize;
                let v = memory.get(pointer)?;
                if v != 0 {
                    let (d1, d2) = u32_to_two_delta(bytecode.addr);
                    memory.sub(pointer + d1 as isize, v)?;
                    memory.sub(pointer + d2 as isize, v)?;
                    memory.set(pointer, 0)?;
                }
            }

            OpCode::MoveStart => {
                pointer += bytecode.delta as isize;
                let val = memory.get(pointer)?;
                if val == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode::MoveAdd => {
                memory.add(pointer + bytecode.delta as isize, mul_val)?;
            }
            OpCode::MoveSub => {
                memory.sub(pointer + bytecode.delta as isize, mul_val)?;
            }

            OpCode::In => {
                pointer += bytecode.delta as isize;
                match stdin.read_exact(&mut stdin_buf) {
                    Ok(_) => memory.set(pointer, stdin_buf[0])?,
                    Err(_) => memory.set(pointer, 0)?,
                }
            }
            OpCode::Out => {
                pointer += bytecode.delta as isize;
                stdout.write(&[memory.get(pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode::JmpIfZero => {
                pointer += bytecode.delta as isize;
                if memory.get(pointer)? == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::JmpIfNotZero => {
                pointer += bytecode.delta as isize;
                if memory.get(pointer)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::PositiveRangeCheckJNZ => {
                pointer += bytecode.delta as isize;
                if (bytecode.val as i8 as isize) >= pointer {
                    println!("deopt");
                }
                if memory.get(pointer)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode::NegativeRangeCheckJNZ => {
                pointer += bytecode.delta as isize;
                if (bytecode.val as i8 as isize) < pointer {
                    println!("deopt");
                }
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
