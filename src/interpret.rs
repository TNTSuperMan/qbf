use std::io::{Write, stdout};

use crate::{bytecode::{Bytecode2, OpCode2}, memory::Memory};

#[inline(always)]
fn u32_to_delta_and_val(val: u32) -> (i16, u8) {
    (
        (val & 0xFFFF) as u16 as i16,
        (val >> 16) as u8,
    )
}

pub fn run(insts: &[Bytecode2], memory: &mut Memory) -> Result<(), String> {
    let mut stdout = stdout().lock();
    let mut pc: usize = 0;
    let mut pointer: isize = 0;
    let mut mul_val: u8 = 0;
    
    loop {
        let bytecode = &insts[pc];
        pointer += bytecode.delta as isize;
        //println!("{},{}",pc,pointer);
        match bytecode.opcode {
            OpCode2::Breakpoint => {
                eprintln!("PC: {}, PTR: {}", pc, pointer);
            }

            OpCode2::SingleAdd => {
                memory.add(pointer, bytecode.val)?;
            }
            OpCode2::SingleSet => {
                memory.set(pointer, bytecode.val)?;
            }
            OpCode2::AddAdd => {
                memory.add(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode2::AddSet => {
                memory.add(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }
            OpCode2::SetAdd => {
                memory.set(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode2::SetSet => {
                memory.set(pointer, bytecode.val)?;
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }

            OpCode2::Shift => {
                let step = bytecode.addr as i32 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
            }
            OpCode2::ShiftAdd => {
                let step = bytecode.val as i8 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.add(pointer, val)?;
            }
            OpCode2::ShiftSet => {
                let step = bytecode.val as i8 as isize;
                while memory.get(pointer)? != 0 {
                    pointer += step;
                }
                let (delta, val) = u32_to_delta_and_val(bytecode.addr);
                pointer += delta as isize;
                memory.set(pointer, val)?;
            }

            OpCode2::MulStart => {
                let val = memory.get(pointer)?;
                if val == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                } else {
                    mul_val = val;
                    memory.set(pointer, 0)?;
                }
            }
            OpCode2::Mul => {
                memory.add(pointer, mul_val.wrapping_mul(bytecode.val))?;
            }
            OpCode2::MulLast => {
                memory.add(pointer, mul_val.wrapping_mul(bytecode.val))?;
                pointer += bytecode.addr as i32 as isize;
            }

            OpCode2::In => {
                memory.set(pointer, 0)?;
            }
            OpCode2::Out => {
                stdout.write(&[memory.get(pointer)?]).map_err(|_| "Runtime Error: Failed to print")?;
            }

            OpCode2::JmpIfZero => {
                if memory.get(pointer)? == 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }
            OpCode2::JmpIfNotZero => {
                if memory.get(pointer)? != 0 {
                    pc = bytecode.addr as usize;
                    continue;
                }
            }

            OpCode2::End => {
                return Ok(());
            }
        }
        pc += 1;
    }
}
